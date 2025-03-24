// fixed_string/src/fx_buf.rs

use super::*;

/// A builder for incrementally constructing a FixedStr of fixed capacity.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FixedStrBuf<const N: usize> {
  buffer: [u8; N],
  len: usize,
}

impl<const N: usize> FixedStrBuf<N> {
  /// Returns the total capacity of the buffer.
  pub const fn capacity(&self) -> usize { N }
  /// Returns the number of bytes remaining in the buffer.
  pub fn remaining(&self) -> usize { N - self.len }
  /// Returns the number of bytes currently in the buffer.
  pub fn len(&self) -> usize { self.len }

  /// Creates a new, empty FixedStrBuf.
  /// 
  /// # Panics
  /// Panics if N == 0.
  pub const fn new() -> Self {
    panic_on_zero(N);
    Self { buffer: [0u8; N], len: 0 }
  }

  /// Attempts to interpret the written portion of the buffer as a valid UTF-8 string.
  ///
  /// Returns an error if the current contents are not valid UTF-8.
  pub fn try_as_str(&self) -> Result<&str, FixedStrError> {
    core::str::from_utf8(self.effective_bytes()).map_err(|_| FixedStrError::InvalidUtf8)
  }

  /// Attempts to append the entire string to the buffer.
  ///
  /// If the string’s byte-length is greater than the remaining capacity,
  /// no data is pushed and an error is returned.
  pub fn try_push_str(&mut self, s: &str) -> Result<(), FixedStrError> {
    let bytes = s.effective_bytes();
    if bytes.len() > self.remaining() {
      return Err(FixedStrError::Overflow { available: self.remaining(), found: bytes.len() });
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
      truncate_utf8_lossy(s.as_bytes(), remaining)
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
  pub fn finalize(mut self) -> Result<FixedStr<N>, FixedStrError> {
    for i in self.len..N {
      self.buffer[i] = 0;
    }  
    Ok(FixedStr::from_bytes(self.buffer))
  }

  /// Clears the builder for reuse, resetting its content to empty.
  pub fn clear(&mut self) {
    self.buffer.fill(0);
    self.len = 0;
  }

  /// Creates a `String` from the effective bytes.
  #[cfg(feature = "std")]
  pub fn to_string_lossy(&self) -> String {
    String::from_utf8_lossy(self.effective_bytes()).into_owned()
  }
}

//******************************************************************************
//  Implementations
//******************************************************************************

impl<const N: usize> fmt::Display for FixedStrBuf<N> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s = match core::str::from_utf8(&self.buffer[..self.len]) {
      Ok(s) => s,
      Err(_) => "<invalid UTF-8>",
    };
    write!(f, "{}", s)
  }
}

impl<const N: usize> fmt::Debug for FixedStrBuf<N> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match str::from_utf8(&self.buffer[..self.len]) {
      Ok(s) => write!(f, "FixedStrBuf<{N}>({:?})", s),
      Err(_) => {
        #[cfg(feature = "std")]
        {
          write!(f, "FixedStrBuf<{}>(<invalid UTF-8>)\n{}", N, FixedStr::<N>::format_hex(&self.buffer[..self.len], 8))
        }
        #[cfg(not(feature = "std"))]
        {
          write!(f, "FixedStrBuf<{}>(<invalid UTF-8>) {:?}", N, &self.buffer[..self.len])
        }
      }
    }
  }
}

impl<const N: usize> EffectiveBytes for FixedStrBuf<N> {
  fn effective_bytes(&self) -> &[u8] {
      self.buffer.effective_bytes()
  }
}

impl<const N: usize> AsRef<[u8]> for FixedStrBuf<N> {
  fn as_ref(&self) -> &[u8] {
    &self.buffer
  }
}

impl<const N: usize> Default for FixedStrBuf<N> {
  fn default() -> Self {
    Self { buffer: [0; N], len: 0 }
  }
}

impl<const N: usize> core::ops::Deref for FixedStrBuf<N> {
  type Target = [u8];
  fn deref(&self) -> &Self::Target {
      &self.buffer
  }
}

/// Creates a `FixedStrBuf` from `FixedStr`
/// 
/// # Panics
/// Panics if N == 0.
impl<const N: usize> From<FixedStr<N>> for FixedStrBuf<N> {
  fn from(fixed: FixedStr<N>) -> Self {
    let buf = copy_into_buffer(&fixed, BufferCopyMode::Truncate).unwrap();
    Self { buffer: buf, len: buf.len() }
  }
}

/// Tries to create a `FixedStrBuf`from `&[u8]`
/// 
/// # Panics
/// Panics if N == 0.
impl<const N: usize> core::convert::TryFrom<&[u8]> for FixedStrBuf<N> {
  type Error = FixedStrError;
  /// Attempts to create a `FixedStr` from a byte slice.
  fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
    let buf = copy_into_buffer(&slice, BufferCopyMode::Exact).unwrap();
    Ok(Self { buffer: buf, len: buf.len() })
  }
}

impl<const N: usize> Hash for FixedStrBuf<N> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    // Only hash bytes up to the first zero (the effective string)
    self.effective_bytes().hash(state);
  }
}

impl<const N: usize> IntoIterator for FixedStrBuf<N> {
  type Item = u8;
  type IntoIter = core::array::IntoIter<u8, N>;

  fn into_iter(self) -> Self::IntoIter {
    core::array::IntoIter::into_iter(self.buffer.into_iter())
  }
}

impl<const N: usize> Ord for FixedStrBuf<N> {
  fn cmp(&self, other: &Self) -> Ordering {
      // Compare only the bytes up to the first zero in each `FixedStr`.
      self.effective_bytes().cmp(&other.effective_bytes())
  }
}

impl<const N: usize> PartialOrd for FixedStrBuf<N> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
      Some(self.cmp(other))
  }
}

impl<const N: usize> PartialEq<[u8]> for FixedStrBuf<N> {
  fn eq(&self, other: &[u8]) -> bool {
    self.effective_bytes() == other.effective_bytes()
  }
}

impl<const N: usize> PartialEq<FixedStrBuf<N>> for [u8] {
  fn eq(&self, other: &FixedStrBuf<N>) -> bool {
    self.effective_bytes() == other.effective_bytes()
  }
}

impl<const N: usize> PartialEq<&[u8]> for FixedStrBuf<N> {
  fn eq(&self, other: &&[u8]) -> bool {
    self.effective_bytes() == other.effective_bytes()
  }
}

impl<const N: usize> PartialEq<FixedStrBuf<N>> for &[u8] {
  fn eq(&self, other: &FixedStrBuf<N>) -> bool {
    self.effective_bytes() == other.effective_bytes()
  }
}

impl<const N: usize> PartialEq<[u8; N]> for FixedStrBuf<N> {
  fn eq(&self, other: &[u8; N]) -> bool {
    self.effective_bytes() == other.effective_bytes()
  }
}

impl<const N: usize> PartialEq<FixedStrBuf<N>> for [u8; N] {
  fn eq(&self, other: &FixedStrBuf<N>) -> bool {
    self.effective_bytes() == other.effective_bytes()
  }
}

impl<const N: usize> PartialEq<FixedStr<N>> for FixedStrBuf<N> {
  fn eq(&self, other: &FixedStr<N>) -> bool {
      self.effective_bytes() == other.effective_bytes()
  }
}

impl<const N: usize> PartialEq<FixedStrBuf<N>> for FixedStr<N> {
  fn eq(&self, other: &FixedStrBuf<N>) -> bool {
      self.effective_bytes() == other.effective_bytes()
  }
}

#[cfg(feature = "std")]
impl<const N: usize> PartialEq<Vec<u8>> for FixedStrBuf<N> {
  fn eq(&self, other: &Vec<u8>) -> bool {
    self.effective_bytes() == other.effective_bytes()
  }
}

#[cfg(feature = "std")]
impl<const N: usize> PartialEq<FixedStrBuf<N>> for Vec<u8> {
  fn eq(&self, other: &FixedStrBuf<N>) -> bool {
    self.effective_bytes() == other.effective_bytes()
  }
}

//******************************************************************************
//  Tests
//******************************************************************************

/// Test module for `FixedStrBuf`.
#[cfg(test)]
pub mod tests {
  use super::*;

  #[test]
  fn test_try_push_str_success() {
      let mut buf = FixedStrBuf::<10>::new();
      assert!(buf.try_push_str("Hello").is_ok());
      assert_eq!(buf.len(), 5);
  }

  #[test]
  fn test_try_push_str_fail() {
      let mut buf = FixedStrBuf::<5>::new();
      // "Hello, world!" is too long to push entirely.
      let result = buf.try_push_str("Hello, world!");
      assert!(result.is_err());
      // The buffer remains unchanged on failure.
      assert_eq!(buf.len(), 0);
  }

  #[test]
  fn test_try_push_char_success() {
      let mut buf = FixedStrBuf::<5>::new();
      assert!(buf.try_push_char('A').is_ok());
      assert_eq!(buf.len(), 1);
  }

  #[test]
  fn test_push_str_lossy() {
      let mut buf = FixedStrBuf::<5>::new();
      // "Hello" fits exactly, so push_str_lossy returns true.
      assert!(buf.push_str_lossy("Hello"));
      // Any additional push will result in truncation.
      let result = buf.push_str_lossy(", world!");
      assert!(!result);
      let fixed: FixedStr<5> = buf.finalize().unwrap();
      assert_eq!(fixed.as_str(), "Hello");
  }

  #[test]
  fn test_finalize_trailing_zeros() {
      let mut buf = FixedStrBuf::<10>::new();
      buf.try_push_str("Hi").unwrap();
      let fixed: FixedStr<10> = buf.finalize().unwrap();
      // The effective string is "Hi" and the rest are zeros.
      assert_eq!(fixed.len(), 2);
      assert_eq!(fixed.as_str(), "Hi");
      assert_eq!(fixed.as_bytes()[2], 0);
  }

  #[test]
  fn test_fixed_str_buf_clear() {
    let mut buf = FixedStrBuf::<10>::new();
    buf.try_push_str("Hello").unwrap();
    assert_eq!(buf.len(), 5);

    buf.clear();
    assert_eq!(buf.len(), 0);
    assert_eq!(&buf, &[0u8; 10]);

    // Can reuse safely
    buf.try_push_str("Rust").unwrap();
    assert_eq!(buf.len(), 4);
    assert_eq!(&buf[..4], b"Rust");
  }
  
  #[cfg(feature = "std")]
  #[test]
  fn test_fixed_str_buf_into_iter() {
    let mut buf = FixedStrBuf::<5>::new();
    buf.try_push_str("Hey").unwrap();
    let bytes: Vec<u8> = buf.into_iter().collect();
    assert_eq!(bytes[..3], *b"Hey");
    assert_eq!(bytes[3..], [0u8; 2]);
  }
}