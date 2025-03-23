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
  pub(crate) data: [u8; N],
}

/// A trait that extracts the effective bytes from the type, i.e. up until the first `\0`.
pub trait EffectiveBytes {
  fn effective_bytes(&self) -> &[u8];
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

  /// Lossy version that creates a valid string.
  pub fn from_slice_lossy(input: &[u8]) -> Self {
    let mut buf = [0u8; N];
    let truncated = Self::truncate_utf8_lossy(input, N);
    buf[..truncated.len()].copy_from_slice(truncated.as_bytes());
    Self { data: buf }
  }

  //****************************************************************************
  //  Modifiers
  //****************************************************************************

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

  //****************************************************************************
  //  Accessors
  //****************************************************************************

  /// Returns the string slice representation.
  ///
  /// # Panics
  /// 
  /// This panics if not valid UTF-8.
  #[track_caller]
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
  pub const fn as_bytes(&self) -> &[u8] {
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

  //****************************************************************************
  //  Helper Functions
  //****************************************************************************

  /// Computes the maximum number of bytes from `input` that can be copied
  /// into a buffer of size `capacity` without splitting a multi-byte UTF‑8 character.
  ///
  /// # Parameters
  /// 
  /// - `input`: The source string.
  /// - `capacity`: The size of the fixed buffer.
  ///
  /// # Returns
  /// 
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
  /// 
  /// - `bytes`: The input byte slice to be truncated.
  /// - `max_len`: The maximum number of bytes to consider from the beginning of `bytes`.
  ///
  /// # Returns
  /// 
  /// A string slice containing a valid UTF‑8 sequence, truncated to a length that does not exceed `max_len`.
  pub fn truncate_utf8_lossy(bytes: &[u8], max_len: usize) -> &str {
    let mut len = max_len.min(bytes.len());
    while len > 0 && str::from_utf8(&bytes[..len]).is_err() {
      len -= 1;
    }
    unsafe { str::from_utf8_unchecked(&bytes[..len]) }
  }

  //****************************************************************************
  //  STD Functions
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
  ///
  /// # Panics
  /// 
  /// This panics if the effective string (up to the first zero)
  /// is not valid UTF‑8.
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
