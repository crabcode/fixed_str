// fixed_string/src/fixed_str.rs

use super::*;

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
///  ```
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct FixedStr<const N: usize> {
  pub(super) data: [u8; N],
}

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
///  ```
impl<const N: usize> FixedStr<N> {
  /// Returns the maximum capacity of the `FixedStr`.
  pub const fn capacity(&self) -> usize { N }
  /// Returns true if the bytes up to the first zero form a valid UTF-8 string.
  pub fn is_valid(&self) -> bool { self.try_as_str().is_ok() }
  /// Returns the number of valid bytes up to the first zero byte.
  pub fn len(&self) -> usize { self.data.effective_bytes().len() }

  //****************************************************************************
  //  Constructors
  //****************************************************************************

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
  /// 
  /// # Panics
  /// 
  /// Panics if N == 0.
  pub fn new(input: &str) -> Self {
    panic_on_zero(N);
    let bytes = input.as_bytes();
    let mut buf = [0u8; N];
    let valid_len = find_valid_utf8_len(bytes, N);
    buf[..valid_len].copy_from_slice(&bytes[..valid_len]);
    Self { data: buf }
  }

  /// Creates a new `FixedStr` at compile time, truncating at the last valid UTF-8 boundary.
  ///
  /// Unlike [`FixedStr::new`], this method does **not** check whether the input fits
  /// fully or whether any characters were truncated.
  ///
  /// If the string contains `\0`, the rest will be truncated.
  /// 
  /// If the string contains multibyte characters near the edge of the buffer,
  /// they will be omitted silently. If no valid boundary is found, the result may be empty.
  ///
  /// # Warning
  ///
  /// This method **does not report truncation**. It is intended for use
  /// in compile-time settings where partial data is acceptable.
  ///
  /// Use [`FixedStr::new`] in runtime contexts for stricter handling.
  /// 
  /// # Panics
  /// 
  /// Panics if N == 0.
  pub const fn new_const(input: &str) -> Self {
    panic_on_zero(N);
    let bytes = input.as_bytes();
    let mut buf = [0u8; N];
    let mut i = 0;
    let len = find_valid_boundary(&bytes, N);

    while i < N && i < len {
      buf[i] = bytes[i];
      i += 1;
    }
    
    Self { data: buf }
  }
  /// Creates a `FixedStr` from a slice.
  ///
  /// If the slice length is less than `N`, only the first `slice.len()` bytes are copied (and the rest remain zero).
  /// If the slice is longer than `N`, only the first `N` bytes are used.
  /// 
  /// If the slice doesn't end on a valid UTF-8 character, the string is truncated.
  pub fn from_slice(input: &[u8]) -> Self {
    panic_on_zero(N);
    let mut buf = [0u8; N];
    let truncated = truncate_utf8_lossy(input, N);
    buf[..truncated.len()].copy_from_slice(truncated.as_bytes());
    Self { data: buf }
  }
  
  /// `from_slice` alternate that stores all bytes without UTF-8 validity check.
  /// 
  /// **Warning:** Does not check UTF-8 validity. Returned `FixedStr` could panic during later use.
  pub fn from_slice_unsafe(slice: &[u8]) -> Self {
    panic_on_zero(N);
    let mut buf = [0u8; N];
    let len = slice.len().min(N);
    buf[..len].copy_from_slice(&slice[..len]);
    Self { data: buf }
  }

  /// Constructs a `FixedStr` from an array of bytes.
  /// 
  /// Truncates the string if invalid UTF-8 data is found.
  pub fn from_bytes(bytes: [u8; N]) -> Self {
    panic_on_zero(N);
    let mut buf = [0u8; N];
    let truncated = truncate_utf8_lossy(&bytes, N);
    buf[..truncated.len()].copy_from_slice(truncated.as_bytes());
    Self { data: buf }
  }
  
  /// `from_slice` alternate that stores all bytes without UTF-8 validity check.
  /// 
  /// **Warning:** Does not check UTF-8 validity. Returned `FixedStr` could panic during later use.
  /// 
  /// # Panics
  /// 
  /// Panics if N == 0.
  pub fn from_bytes_unsafe(bytes: [u8; N]) -> Self {
    panic_on_zero(N);
    let mut buf = [0u8; N];
    let len = bytes.len().min(N);
    buf[..len].copy_from_slice(&bytes[..len]);
    Self { data: buf }
  }


  //****************************************************************************
  //  Modifiers
  //****************************************************************************

  /// Updates the `FixedStr` with a new value, replacing the current content.
  ///
  /// The input string is copied into the internal buffer. If the input exceeds
  /// the capacity, an error is thrown. If the input is shorter than the capacity,
  /// the remaining bytes are set to zero.
  /// 
  /// **Warning:** if `input` contains `\0`, the rest will be truncated.
  pub fn set(&mut self, input: &str) -> Result<(), FixedStrError> {
    let bytes = input.effective_bytes();
    let len = bytes.len();
    if len > N {
      return Err(FixedStrError::Overflow { available: N, found: len });
    }
    let mut buf = [0u8; N];
    buf[..len].copy_from_slice(&bytes);
    self.data = buf;
    Ok(())
  }

  /// Truncates overflowing bytes down to the last valid UTF-8 string.
  /// 
  /// **Warning:** if `input` contains `\0`, the rest will be truncated.
  /// 
  /// # Examples
  /// 
  /// use fixed_string::FixedStr;
  ///
  /// let mut fs = FixedStr::<5>::new("Hello");
  /// fs.set_lossy("World!");
  /// // "World!" is truncated to "World" because the capacity is 5 bytes.
  /// assert_eq!(fs.as_str(), "World");
  pub fn set_lossy(&mut self, input: &str) {
    let valid = truncate_utf8_lossy(input.as_bytes(), N);
    let mut buf = [0u8; N];
    buf[..valid.len()].copy_from_slice(&valid.as_bytes());
    self.data = buf;
  }

  /// Clears the `FixedStr`, setting all bytes to zero.
  pub fn clear(&mut self) {
    self.data = [0u8; N];
  }

  //****************************************************************************
  //  Accessors
  //****************************************************************************

  /// Returns the string slice representation.
  #[track_caller]
  pub fn as_str(&self) -> &str {
    truncate_utf8_lossy(self, N)
  }

  /// Attempts to interpret the stored bytes as a UTF‑8 string.
  ///
  /// Returns an error if the data up to the first zero byte is not valid UTF‑8.
  pub fn try_as_str(&self) -> Result<&str, FixedStrError> {
    str::from_utf8(self.effective_bytes()).map_err(|_| FixedStrError::InvalidUtf8)
  }

  /// Returns the raw bytes stored in the `FixedStr`.
  pub const fn as_bytes(&self) -> &[u8] {
    &self.data
  }

  #[cfg(feature = "const_mut_refs")]
  /// Returns the raw bytes stored in the `FixedStr` as `mut`
  pub const fn as_mut_bytes(&mut self) -> &mut [u8] {
    &mut self.data
  }

  #[cfg(not(feature = "const_mut_refs"))]
  /// Returns the raw bytes stored in the `FixedStr` as `mut`
  pub fn as_mut_bytes(&mut self) -> &mut [u8] {
    &mut self.data
  }

  /// Returns an iterator that goes through the full byte
  /// array instead of terminating at the first `\0`.
  pub fn byte_iter(&self) -> impl Iterator<Item = u8> + '_ {
    self.data.iter().copied()
  }

  //****************************************************************************
  //  std Functions
  //****************************************************************************

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
  #[cfg(feature = "std")]
  pub fn into_string(self) -> String {
    self.as_str().to_string()
  }

  /// Attempts to convert the `FixedStr` to an owned String.
  #[cfg(feature = "std")]
  pub fn try_into_string(self) -> Result<String, FixedStrError> {
    self.try_as_str().map(str::to_string)
  }

  /// Converts the `FixedStr` to an owned String in a lossy manner,
  /// replacing any invalid UTF‑8 sequences with the Unicode replacement character.
  #[cfg(feature = "std")]
  pub fn to_string_lossy(&self) -> String {
    String::from_utf8_lossy(&self.data[..self.len()]).into_owned()
  }
}
