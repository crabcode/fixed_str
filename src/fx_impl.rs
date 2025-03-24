// fixed_string/src/fx_impl.rs

use super::*;
use core::usize;

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

impl<const N: usize> Default for FixedStr<N> {
  fn default() -> Self {
    Self { data: [0; N] }
  }
}

impl<const N: usize> core::ops::Deref for FixedStr<N> {
  type Target = [u8];
  fn deref(&self) -> &Self::Target {
      &self.data
  }
}

impl<const N: usize> core::ops::DerefMut for FixedStr<N> {
  fn deref_mut(&mut self) -> &mut Self::Target {
      &mut self.data
  }
}

impl<const N: usize> core::convert::TryFrom<&[u8]> for FixedStr<N> {
  type Error = FixedStrError;
  /// Attempts to create a `FixedStr` from a byte slice.
  /// 
  /// # Error
  /// - `WrongLength`: Thrown if the slice's effective bytes are longer than N.
  /// - `InvalidUtf8`: Thrown if the resulting string isn't valid UTF-8.
  /// 
  /// Returns `FixedStr` if successful.
  /// 
  /// # Panics
  /// Panics if N == 0.
  fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
    let buf = copy_into_buffer(&slice.effective_bytes(), BufferCopyMode::Exact).unwrap();
    let result = Self { data: buf };
    match result.is_valid() {
      true => Ok(result),
      false => Err(FixedStrError::InvalidUtf8),
    }
  }
}

impl<const N: usize> From<&str> for FixedStr<N> {
  /// **Warning:** If the input string contains `\0` or invaliid UTF-8 code, the rest will be truncated.
  fn from(s: &str) -> Self {
    Self::new(s)
  }
}

impl<const N: usize> Hash for FixedStr<N> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    // Only hash bytes up to the first zero (the effective string)
    self.effective_bytes().hash(state);
  }
}

impl<const N: usize> IntoIterator for FixedStr<N> {
  type Item = u8;
  type IntoIter = EffectiveBytesIter<N>;

  fn into_iter(self) -> Self::IntoIter {
    EffectiveBytesIter { data: self.data, index: 0, len: self.len() }
  }
}

impl<const N: usize> Ord for FixedStr<N> {
  fn cmp(&self, other: &Self) -> Ordering {
      // Compare only the bytes up to the first zero in each `FixedStr`.
      self.effective_bytes().cmp(&other.effective_bytes())
  }
}

impl<const N: usize> PartialOrd for FixedStr<N> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
      Some(self.cmp(other))
  }
}

impl<const N: usize> PartialEq<&str> for FixedStr<N> {
  fn eq(&self, other: &&str) -> bool {
    self.effective_bytes() == other.effective_bytes()
  }
}

impl<const N: usize> PartialEq<FixedStr<N>> for &str {
  fn eq(&self, other: &FixedStr<N>) -> bool {
    self.effective_bytes() == other.effective_bytes()
  }
}

impl<const N: usize> PartialEq<[u8]> for FixedStr<N> {
  fn eq(&self, other: &[u8]) -> bool {
    self.effective_bytes() == other.effective_bytes()
  }
}

impl<const N: usize> PartialEq<FixedStr<N>> for [u8] {
  fn eq(&self, other: &FixedStr<N>) -> bool {
    self.effective_bytes() == other.effective_bytes()
  }
}

impl<const N: usize> PartialEq<&[u8]> for FixedStr<N> {
  fn eq(&self, other: &&[u8]) -> bool {
    self.effective_bytes() == other.effective_bytes()
  }
}

impl<const N: usize> PartialEq<FixedStr<N>> for &[u8] {
  fn eq(&self, other: &FixedStr<N>) -> bool {
    self.effective_bytes() == other.effective_bytes()
  }
}

impl<const N: usize> PartialEq<[u8; N]> for FixedStr<N> {
  fn eq(&self, other: &[u8; N]) -> bool {
    self.effective_bytes() == other.effective_bytes()
  }
}

impl<const N: usize> PartialEq<FixedStr<N>> for [u8; N] {
  fn eq(&self, other: &FixedStr<N>) -> bool {
    self.effective_bytes() == other.effective_bytes()
  }
}

//******************************************************************************
//  std Implementations
//******************************************************************************

/// Implementations for the Standard Library.
#[cfg(feature = "std")]
pub mod std_ext {
  use super::*;

  impl<const N: usize> PartialEq<Vec<u8>> for FixedStr<N> {
    fn eq(&self, other: &Vec<u8>) -> bool {
      self.effective_bytes() == other.effective_bytes()
    }
  }

  impl<const N: usize> PartialEq<FixedStr<N>> for Vec<u8> {
    fn eq(&self, other: &FixedStr<N>) -> bool {
      self.effective_bytes() == other.effective_bytes()
    }
  }

  impl<const N: usize> PartialEq<String> for FixedStr<N> {
    fn eq(&self, other: &String) -> bool {
      self.effective_bytes() == other.effective_bytes()
    }
  }

  impl<const N: usize> PartialEq<FixedStr<N>> for String {
    fn eq(&self, other: &FixedStr<N>) -> bool {
      self.effective_bytes() == other.effective_bytes()
    }
  }

  impl<const N: usize> From<String> for FixedStr<N> {
    fn from(s: String) -> Self {
      Self::new(&s)
    }
  }

  impl<const N: usize> From<FixedStr<N>> for String {
    fn from(fs: FixedStr<N>) -> Self {
      fs.into_string()
    }
  }

  impl<const N: usize> From<&FixedStr<N>> for String {
    fn from(fs: &FixedStr<N>) -> Self {
      fs.into_string()
    }
  }
}

//******************************************************************************
//  Tests
//******************************************************************************

#[cfg(test)]
mod impl_tests {
  use super::*;

  #[test]
  fn test_set_success() {
      // Test that FixedStr::set successfully replaces the content when it fits.
      let mut fixed = FixedStr::<10>::new("Hello");
      assert_eq!(fixed.as_str(), "Hello");
      fixed.set("Rust").unwrap();
      assert_eq!(fixed.as_str(), "Rust");
  }

  #[test]
  #[should_panic]
  fn test_set_overflow() {
      // Test that FixedStr::set panics (via the Exact mode) when the input is too long.
      let mut fixed = FixedStr::<5>::new("Hi");
      // This should panic because "Hello, world!" is longer than 5 bytes.
      fixed.set("Hello, world!").unwrap();
  }

  #[test]
  fn test_set_lossy() {
      // Test that FixedStr::set_lossy truncates the input safely.
      let mut fixed = FixedStr::<5>::new("Hello");
      fixed.set_lossy("Rustaceans");
      // "Rustaceans" is too long for 5 bytes; it is expected to be truncated safely,
      // for example to "Rusta" (assuming that is the valid UTF‑8 prefix).
      assert_eq!(fixed.as_str(), "Rusta");
  }

  #[test]
  fn test_ordering() {
      // Test ordering between FixedStr values.
      let a = FixedStr::<10>::new("Apple");
      let b = FixedStr::<10>::new("Banana");
      let c = FixedStr::<10>::new("Apple");
      assert!(a < b);
      assert_eq!(a, c);
  }

  #[cfg(feature = "std")]
  #[test]
  fn test_from_string_and_into_string() {
      // Test conversion from String to FixedStr and back.
      let s = String::from("Hello");
      let fixed: FixedStr<10> = s.clone().into();
      assert_eq!(fixed.as_str(), "Hello");
      let s2: String = fixed.into();
      assert_eq!(s2, "Hello");
  }

  #[test]
  fn test_as_mut_bytes() {
      // Test that modifying the mutable bytes directly affects the effective string.
      let mut fixed = FixedStr::<10>::new("Hello");
      {
          let bytes = fixed.as_mut_bytes();
          // Change the first byte from 'H' to 'J'
          bytes[0] = b'J';
      }
      assert_eq!(fixed.as_str(), "Jello");
  }

}