//! A fixed–length, null–padded UTF‑8 string type.
//!
//! `FixedStr<N>` stores exactly N bytes and pads or truncates as needed.
//!
//! # Note on UTF‑8 Safety
//! When using `new`, if the input is longer than N, it is safely truncated at the last valid UTF‑8 boundary.
//! The `new_const` method does not perform this check and should be used with care.

#![cfg_attr(not(feature = "std"), no_std)]
use core::{fmt, str};

#[cfg(feature = "std")]
use std::string::String;

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(feature = "binrw")]
use binrw::{BinRead, BinWrite};
#[cfg(feature = "binrw")]
use binrw::io::{Read, Seek, Write};

//******************************************************************************
//  FixedStr
//******************************************************************************

/// A fixed–length string with a constant size of `N` bytes.
///
/// Internally, the string is stored in a `[u8; N]` array.
/// Unused bytes are left as zeros. When converting to a `&str`,
/// the first `0` byte is considered the end of the string.
///
/// # Examples
/// ```
/// use fixed_string::FixedStr;
///
/// let fs = FixedStr::<5>::new("Hello");
/// assert_eq!(fs.as_str(), "Hello");
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct FixedStr<const N: usize> {
  data: [u8; N],
}

impl<const N: usize> FixedStr<N> {
  /// Creates a new `FixedStr` from the given input string.
  ///
  /// The input is converted to bytes and copied into a fixed–size buffer.
  /// If the input is longer than `N`, it is safely truncated at the last valid UTF‑8 boundary.
  /// If the input is shorter than `N`, the remainder is filled with zeros.
  pub fn new(input: &str) -> Self {
    let mut buf = [0u8; N];
    let bytes = input.as_bytes();
    let valid_len = if bytes.len() > N {
      // Find the maximum valid UTF‑8 boundary within N bytes.
      let mut valid = 0;
      for i in (0..=N).rev() {
        if input.is_char_boundary(i) {
          valid = i;
          break;
        }
      }
      valid
    } else {
      bytes.len()
    };
    buf[..valid_len].copy_from_slice(&bytes[..valid_len]);
    Self { data: buf }
  }

  /// Creates a new `FixedStr` in a const context.
  ///
  /// **Warning:** This method does not perform UTF‑8 boundary checking,
  /// so if the input is longer than `N` and is truncated in the middle of a character,
  /// the resulting string may be invalid UTF‑8.
  pub const fn new_const(input: &str) -> Self {
    let bytes = input.as_bytes();
    let mut buf = [0u8; N];
    let mut i = 0;
    while i < N && i < bytes.len() {
      buf[i] = bytes[i];
      i += 1;
    }
    Self { data: buf }
  }

  /// Returns the number of valid bytes up to the first zero byte.
  pub fn len(&self) -> usize {
    self.data.iter().position(|&b| b == 0).unwrap_or(N)
  }

  /// Returns the maximum capacity of the FixedStr.
  pub fn capacity(&self) -> usize {
    N
  }

  /// Attempts to interpret the stored bytes as a UTF‑8 string.
  ///
  /// Returns an error if the data up to the first zero byte is not valid UTF‑8.
  pub fn try_as_str(&self) -> Result<&str, FixedStrError> {
    let end = self.len();
    str::from_utf8(&self.data[..end]).map_err(|_| FixedStrError::InvalidUtf8)
  }

  /// Returns the string slice representation.
  ///
  /// Panics if not valid UTF-8.
  pub fn as_str(&self) -> &str {
    self.try_as_str().expect("<invalid utf-8>")
  }

  /// Returns the raw bytes stored in the `FixedStr`.
  pub fn as_bytes(&self) -> &[u8] {
    &self.data
  }

  /// Returns a hex–encoded string of the entire fixed buffer.
  ///
  /// Each byte is represented as a two–digit uppercase hex number.
  #[cfg(feature = "std")]
  pub fn as_hex(&self) -> String {
    format_hex(&self.data, self.data.len())
  }

  /// Returns a formatted hex dump of the data.
  ///
  /// The bytes are grouped in 8–byte chunks, with each chunk on a new line.
  #[cfg(feature = "std")]
  pub fn as_hex_dump(&self) -> String {
    format_hex(&self.data, 8)
  }

  /// Constructs a `FixedStr` from an array of bytes.
  pub const fn from_bytes(bytes: [u8; N]) -> Self {
    Self { data: bytes }
  }

  /// Creates a `FixedStr` from a slice.
  ///
  /// If the slice length is less than `N`, only the first `slice.len()` bytes are copied (and the rest remain zero).
  /// If the slice is longer than `N`, only the first `N` bytes are used.
  pub fn from_slice(slice: &[u8]) -> Self {
    let mut buf = [0u8; N];
    let len = slice.len().min(N);
    buf[..len].copy_from_slice(&slice[..len]);
    Self { data: buf }
  }
}

impl<const N: usize> fmt::Debug for FixedStr<N> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // Use the same boundary as try_as_str for consistency.
    match self.try_as_str() {
      Ok(s) => write!(f, "{:?}", s),
      Err(_) => {
        #[cfg(feature = "std")]
        {
          write!(f, "<invalid UTF-8>\n{}", self.as_hex_dump())
        }
        #[cfg(not(feature = "std"))]
        {
          write!(f, "<invalid UTF-8> {:?}", self.as_bytes())
        }
      },
    }
  }
}

impl<const N: usize> fmt::Display for FixedStr<N> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

//******************************************************************************
//  Core Implementations
//******************************************************************************

impl<const N: usize> Default for FixedStr<N> {
  fn default() -> Self {
    Self { data: [0; N] }
  }
}

impl<const N: usize> AsRef<[u8]> for FixedStr<N> {
  fn as_ref(&self) -> &[u8] {
    &self.data
  }
}

impl<const N: usize> core::ops::Deref for FixedStr<N> {
  type Target = [u8];
  fn deref(&self) -> &Self::Target {
      &self.data
  }
}

impl<const N: usize> From<&str> for FixedStr<N> {
  fn from(s: &str) -> Self {
    Self::new(s)
  }
}

impl<const N: usize> core::convert::TryFrom<&[u8]> for FixedStr<N> {
  type Error = FixedStrError;
  /// Attempts to create a `FixedStr` from a byte slice.
  ///
  /// The slice must be exactly `N` bytes long, or else an error is returned.
  fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
    if slice.len() != N {
      return Err(FixedStrError::WrongLength { expected: N, found: slice.len() });
    }
    let mut buf = [0u8; N];
    buf.copy_from_slice(slice);
    Ok(Self { data: buf })
  }
}

impl<const N: usize> IntoIterator for FixedStr<N> {
  type Item = u8;
  type IntoIter = core::array::IntoIter<u8, N>;

  fn into_iter(self) -> Self::IntoIter {
    core::array::IntoIter::into_iter(self.data.into_iter())
  }
}

impl<const N: usize> PartialEq<&str> for FixedStr<N> {
  fn eq(&self, other: &&str) -> bool {
    self.as_str() == *other
  }
}

impl<const N: usize> PartialEq<FixedStr<N>> for &str {
  fn eq(&self, other: &FixedStr<N>) -> bool {
    *self == other.as_str()
  }
}

//******************************************************************************
//  Feature Implementations
//******************************************************************************

#[cfg(feature = "std")]
impl<const N: usize> PartialEq<String> for FixedStr<N> {
  fn eq(&self, other: &String) -> bool {
    self.as_str() == other.as_str()
  }
}

#[cfg(feature = "std")]
impl<const N: usize> PartialEq<FixedStr<N>> for String {
  fn eq(&self, other: &FixedStr<N>) -> bool {
    self.as_str() == other.as_str()
  }
}

#[cfg(feature = "std")]
impl<const N: usize> From<String> for FixedStr<N> {
  fn from(s: String) -> Self {
    Self::new(&s)
  }
}

#[cfg(feature = "binrw")]
impl<const N: usize> BinRead for FixedStr<N> {
  type Args<'a> = ();

  fn read_options<R: Read + Seek>(reader: &mut R, _endian: binrw::Endian, _args: Self::Args<'_>) -> binrw::BinResult<Self> {
    let mut buf = [0u8; N];
    reader.read_exact(&mut buf)?;
    Ok(Self { data: buf })
  }
}

#[cfg(feature = "binrw")]
impl<const N: usize> BinWrite for FixedStr<N> {
  type Args<'a> = ();

  fn write_options<W: Write + Seek>(&self, writer: &mut W, _endian: binrw::Endian, _args: Self::Args<'_>) -> binrw::BinResult<()> {
    writer.write_all(&self.data)?;
    Ok(())
  }
}

//******************************************************************************
//  FixedStrError
//******************************************************************************

/// Custom error type for FixedStr conversions.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FixedStrError {
  WrongLength { expected: usize, found: usize },
  InvalidUtf8,
}

impl fmt::Debug for FixedStrError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      match self {
          Self::WrongLength { expected, found } => {
            write!(f, "WrongLength: expected {}, found {}", expected, found)
          },
          Self::InvalidUtf8 => write!(f, "InvalidUtf8"),
      }
  }
}

impl fmt::Display for FixedStrError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      match self {
          Self::WrongLength { expected, found } => {
            write!(f, "Wrong length: expected {} bytes, found {} bytes", expected, found)
          },
          Self::InvalidUtf8 => write!(f, "Invalid UTF-8"),
      }
  }
}

#[cfg(feature = "std")]
impl std::error::Error for FixedStrError {}

//******************************************************************************
//  Helper Functions
//******************************************************************************

pub fn truncate_utf8_lossy(bytes: &[u8], max_len: usize) -> &str {
  let mut len = max_len.min(bytes.len());
  while len > 0 && str::from_utf8(&bytes[..len]).is_err() {
    len -= 1;
  }
  unsafe { str::from_utf8_unchecked(&bytes[..len]) }
}

#[cfg(feature = "std")]
fn format_hex(bytes: &[u8], group: usize) -> String {
  bytes
    .chunks(group)
    .map(|chunk| chunk.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "))
    .collect::<Vec<_>>()
    .join("\n")
}