//! A fixed–length, null–padded UTF‑8 string type.
//!
//! `FixedStr<N>` stores exactly N bytes and pads or truncates as needed.
//!
//! # Note on UTF‑8 Safety
//! When using `new`, if the input is longer than N, it is safely truncated at the last valid UTF‑8 boundary.
//! The `new_const` method does not perform this check and should be used with care.

#![deny(missing_docs)]

#![cfg_attr(not(feature = "std"), no_std)]
use core::{fmt, str, borrow::Borrow, cmp::Ordering, hash::{Hash, Hasher}};

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

  //**********************************************
  //  FixedStr::Constructors
  //**********************************************

  /// Creates a new `FixedStr` from the given input string.
  ///
  /// The input is converted to bytes and copied into a fixed–size buffer.
  /// If the input is longer than the capacity, it is safely truncated at the
  /// last valid UTF‑8 boundary. If the input is shorter, the remaining bytes
  /// are filled with zeros.
  ///
  /// # Examples
  /// ```
  /// use fixed_string::FixedStr;
  ///
  /// // "Hello" fits exactly in a buffer of 5 bytes.
  /// let fs = FixedStr::<5>::new("Hello");
  /// assert_eq!(fs.as_str(), "Hello");
  /// 
  /// // "Hello, World!" is truncated safely to "Hello".
  /// let fs = FixedStr::<5>::new("Hello, World!");
  /// assert_eq!(fs.as_str(), "Hello");
  /// ```
  pub fn new(input: &str) -> Self {
    let mut buf = [0u8; N];
    let valid_len = Self::compute_valid_len(input, N);
    buf[..valid_len].copy_from_slice(&input.as_bytes()[..valid_len]);
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

  //**********************************************
  //  FixedStr::Modifiers
  //**********************************************

  /// Updates the `FixedStr` with a new value, replacing the current content.
  ///
  /// The input string is copied into the internal buffer. If the input exceeds
  /// the capacity, it is truncated at the last valid UTF‑8 boundary so that the
  /// resulting string remains valid UTF‑8. If the input is shorter than the capacity,
  /// the remaining bytes are set to zero.
  ///
  /// # Examples
  /// ```
  /// use fixed_string::FixedStr;
  ///
  /// let mut fs = FixedStr::<5>::new("Hello");
  /// fs.set("World!");
  /// // "World!" is truncated to "World" because the capacity is 5 bytes.
  /// assert_eq!(fs.as_str(), "World");
  /// ```
  pub fn set(&mut self, input: &str) {
    let valid_len = Self::compute_valid_len(input, N);
    let mut buf = [0u8; N];
    buf[..valid_len].copy_from_slice(&input.as_bytes()[..valid_len]);
    self.data = buf;
  }

  /// Clears the `FixedStr`, setting all bytes to zero.
  pub fn clear(&mut self) {
    self.data = [0u8; N];
  }

  //**********************************************
  //  FixedStr::Accessors
  //**********************************************

  /// Returns the string slice representation.
  ///
  /// Panics if not valid UTF-8.
  pub fn as_str(&self) -> &str {
    self.try_as_str().expect("<invalid utf-8>")
  }

  /// Attempts to interpret the stored bytes as a UTF‑8 string.
  ///
  /// Returns an error if the data up to the first zero byte is not valid UTF‑8.
  pub fn try_as_str(&self) -> Result<&str, FixedStrError> {
    let end = self.len();
    str::from_utf8(&self.data[..end]).map_err(|_| FixedStrError::InvalidUtf8)
  }

  /// Returns the raw bytes stored in the `FixedStr`.
  pub fn as_bytes(&self) -> &[u8] {
    &self.data
  }

  /// Returns the maximum capacity of the `FixedStr`.
  pub fn capacity(&self) -> usize {
    N
  }

  /// Returns true if the bytes up to the first zero form a valid UTF-8 string.
  pub fn is_valid(&self) -> bool {
    self.try_as_str().is_ok()
  }

  /// Returns the number of valid bytes up to the first zero byte.
  pub fn len(&self) -> usize {
    self.data.iter().position(|&b| b == 0).unwrap_or(N)
  }

  //**********************************************
  //  FixedStr::Helper Functions
  //**********************************************

  /// Computes the maximum number of bytes from `input` that can be copied
  /// into a buffer of size `capacity` without splitting a multi-byte UTF‑8 character.
  ///
  /// # Parameters
  /// - `input`: The source string.
  /// - `capacity`: The size of the fixed buffer.
  ///
  /// # Returns
  /// The number of bytes to copy, ensuring that the last byte is on a valid UTF‑8 boundary.
  fn compute_valid_len(input: &str, capacity: usize) -> usize {
    let bytes = input.as_bytes();
    if bytes.len() > capacity {
      // Find the last valid UTF‑8 boundary within the capacity.
      for i in (0..=capacity).rev() {
        if input.is_char_boundary(i) {
          return i;
        }
      }
      // Fallback: should never reach here because 0 is always a boundary.
      0
    } else {
      bytes.len()
    }
  }
  
  /// Truncates a byte slice to a valid UTF‑8 string within a maximum length.
  ///
  /// # Parameters
  /// - `bytes`: The input byte slice to be truncated.
  /// - `max_len`: The maximum number of bytes to consider from the beginning of `bytes`.
  ///
  /// # Returns
  /// A string slice containing a valid UTF‑8 sequence, truncated to a length that does not exceed `max_len`.
  pub fn truncate_utf8_lossy(bytes: &[u8], max_len: usize) -> &str {
    let mut len = max_len.min(bytes.len());
    while len > 0 && str::from_utf8(&bytes[..len]).is_err() {
      len -= 1;
    }
    unsafe { str::from_utf8_unchecked(&bytes[..len]) }
  }

  //**********************************************
  //  FixedStr::Feature Functions
  //**********************************************

  /// Formats a byte array into custom chunks
  #[cfg(feature = "std")]
  pub fn format_hex(bytes: &[u8], group: usize) -> String {
    bytes
      .chunks(group)
      .map(|chunk| chunk.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "))
      .collect::<Vec<_>>()
      .join("\n")
  }
  
  /// Returns a hex–encoded string of the entire fixed buffer.
  ///
  /// Each byte is represented as a two–digit uppercase hex number.
  #[cfg(feature = "std")]
  pub fn as_hex(&self) -> String {
    Self::format_hex(&self.data, self.data.len())
  }

  /// Returns a formatted hex dump of the data.
  ///
  /// The bytes are grouped in 8–byte chunks, with each chunk on a new line.
  #[cfg(feature = "std")]
  pub fn as_hex_dump(&self) -> String {
    Self::format_hex(&self.data, 8)
  }

  /// Converts the `FixedStr` to an owned String.
  ///
  /// # Panics
  /// This panics if the effective string (up to the first zero)
  /// is not valid UTF‑8.
  #[cfg(feature = "std")]
  pub fn into_string(self) -> String {
    self.as_str().to_string()
  }

  /// Converts the `FixedStr` to an owned String in a lossy manner,
  /// replacing any invalid UTF‑8 sequences with the Unicode replacement character.
  #[cfg(feature = "std")]
  pub fn to_string_lossy(&self) -> String {
    String::from_utf8_lossy(&self.data[..self.len()]).into_owned()
  }
}

//******************************************************************************
//  Core Implementations
//******************************************************************************

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

impl<const N: usize> AsRef<str> for FixedStr<N> {
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

impl<const N: usize> Borrow<str> for FixedStr<N> {
  fn borrow(&self) -> &str {
    self.as_str()
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

impl<const N: usize> Hash for FixedStr<N> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    // Only hash bytes up to the first zero (the effective string)
    self.data[..self.len()].hash(state);
  }
}

impl<const N: usize> IntoIterator for FixedStr<N> {
  type Item = u8;
  type IntoIter = core::array::IntoIter<u8, N>;

  fn into_iter(self) -> Self::IntoIter {
    core::array::IntoIter::into_iter(self.data.into_iter())
  }
}

impl<const N: usize> Ord for FixedStr<N> {
  fn cmp(&self, other: &Self) -> Ordering {
      // Compare only the bytes up to the first zero in each `FixedStr`.
      let self_slice = &self.data[..self.len()];
      let other_slice = &other.data[..other.len()];
      self_slice.cmp(other_slice)
  }
}

impl<const N: usize> PartialOrd for FixedStr<N> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
      Some(self.cmp(other))
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

#[cfg(feature = "std")]
impl<const N: usize> From<FixedStr<N>> for String {
  fn from(fs: FixedStr<N>) -> Self {
    fs.into_string()
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
//  FixedStrBuf
//******************************************************************************

/// A builder for incrementally constructing a FixedStr of fixed capacity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedStrBuf<const N: usize> {
  buffer: [u8; N],
  len: usize,
}

impl<const N: usize> FixedStrBuf<N> {
  /// Creates a new, empty FixedStrBuf.
  pub const fn new() -> Self {
    Self {
      buffer: [0u8; N],
      len: 0,
    }
  }

  /// Returns the number of bytes currently in the buffer.
  pub fn len(&self) -> usize {
    self.len
  }

  /// Returns the total capacity of the buffer.
  pub fn capacity(&self) -> usize {
    N
  }

  /// Returns the number of bytes remaining in the buffer.
  pub fn remaining(&self) -> usize {
    N - self.len
  }

  /// Attempts to append the entire string to the buffer.
  ///
  /// If the string’s byte-length is greater than the remaining capacity,
  /// no data is pushed and an error is returned.
  pub fn try_push_str(&mut self, s: &str) -> Result<(), FixedStrError> {
    let bytes = s.as_bytes();
    if bytes.len() > self.remaining() {
      return Err(FixedStrError::WrongLength { expected: self.remaining(), found: bytes.len() });
    }
    self.buffer[self.len..self.len + bytes.len()].copy_from_slice(bytes);
    self.len += bytes.len();
    Ok(())
  }

  /// Attempts to append a single character to the buffer.
  ///
  /// Returns an error if the character’s UTF‑8 representation doesn’t fit.
  pub fn try_push_char(&mut self, c: char) -> Result<(), FixedStrError> {
    let mut buf = [0u8; 4];
    let s = c.encode_utf8(&mut buf);
    self.try_push_str(s)
  }

  /// Appends as many complete UTF‑8 characters from `s` as possible.
  ///
  /// If the entire string fits, it returns true. If not, it pushes only
  /// the valid initial segment and returns false.
  pub fn push_str_lossy(&mut self, s: &str) -> bool {
    let remaining = self.remaining();
    let valid = if s.len() > remaining {
      FixedStr::<N>::truncate_utf8_lossy(s.as_bytes(), remaining)
    } else {
      s
    };
    
    let bytes = valid.as_bytes();
    if bytes.len() > 0 {
      self.buffer[self.len..self.len + bytes.len()].copy_from_slice(bytes);
      self.len += bytes.len();
    }

    bytes.len() == s.len()
  }

  /// Finalizes the builder into a FixedStr.
  ///
  /// This method zeros out any unused bytes in the buffer.
  pub fn into_fixed(mut self) -> FixedStr<N> {
    self.buffer.fill(0);
    FixedStr::from_bytes(self.buffer)
  }

  /// Clears the builder for reuse, resetting its content to empty.
  pub fn clear(&mut self) {
    self.buffer.fill(0);
    self.len = 0;
  }
}

impl<const N: usize> fmt::Display for FixedStrBuf<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match core::str::from_utf8(&self.buffer[..self.len]) {
            Ok(s) => s,
            Err(_) => "<invalid UTF-8>",
        };
        write!(f, "{}", s)
    }
}

//******************************************************************************
//  FixedStrError
//******************************************************************************

/// Custom error type for `FixedStr` conversions.
#[derive(Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FixedStrError {
  /// Returned when the length of the input does not match the expected size.
  ///
  /// This usually happens when converting from a byte slice or building a
  /// `FixedStrBuf` where the provided input exceeds capacity.
  ///
  /// - `expected`: The expected length in bytes.
  /// - `found`: The actual length of the provided input.
  WrongLength {
    /// The expected length in bytes.
    expected: usize,
    /// The actual length of the provided input.
    found: usize,
  },
  /// Returned when the byte content could not be parsed as valid UTF-8.
  InvalidUtf8,
  /// Returned when a string is longer than the capacity.
  Truncated,
}

impl fmt::Debug for FixedStrError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      match self {
          Self::WrongLength { expected, found } => {
            write!(f, "WrongLength: expected {}, found {}", expected, found)
          },
          Self::InvalidUtf8 => write!(f, "InvalidUtf8"),
          Self::Truncated => write!(f, "Truncated"),
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
          Self::Truncated => write!(f, "Truncated"),
      }
  }
}

#[cfg(feature = "std")]
impl std::error::Error for FixedStrError {}


//******************************************************************************
//  Serde Serialization
//******************************************************************************

#[cfg(feature = "serde")]
mod serde_impl {
  use super::*;
  use serde::{Serialize, Serializer, Deserialize, Deserializer};
  use serde::de::{Visitor, Error as DeError};
  use core::fmt;

  impl<const N: usize> Serialize for FixedStr<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
      let s = self.to_string_lossy();
      serializer.serialize_str(&s)
    }
  }

  struct FixedStrVisitor<const N: usize>;

  impl<'de, const N: usize> Visitor<'de> for FixedStrVisitor<N> {
    type Value = FixedStr<N>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      write!(formatter, "a string of at most {} bytes", N)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where E: DeError {
      Ok(FixedStr::new(value))
    }
  }

  impl<'de, const N: usize> Deserialize<'de> for FixedStr<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
      deserializer.deserialize_str(FixedStrVisitor::<N>)
    }
  }
}