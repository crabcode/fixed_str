// fixed_string/src/impls.rs

use super::*;

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

impl<const N: usize> EffectiveBytes for FixedStr<N> {
  fn effective_bytes(&self) -> &[u8] {
      &self[..self.len()]
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

impl<const N: usize> From<&str> for FixedStr<N> {
  fn from(s: &str) -> Self {
    Self::new(s)
  }
}

impl<const N: usize> core::convert::TryFrom<&[u8]> for FixedStr<N> {
  type Error = FixedStrError;
  /// Attempts to create a `FixedStr` from a byte slice.
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
    self.effective_bytes().hash(state);
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
    self.as_str() == *other
  }
}

impl<const N: usize> PartialEq<FixedStr<N>> for &str {
  fn eq(&self, other: &FixedStr<N>) -> bool {
    *self == other.as_str()
  }
}

impl<const N: usize> PartialEq<[u8]> for FixedStr<N> {
  fn eq(&self, other: &[u8]) -> bool {
    self.as_bytes() == other
  }
}

impl<const N: usize> PartialEq<FixedStr<N>> for [u8] {
  fn eq(&self, other: &FixedStr<N>) -> bool {
    self == other.effective_bytes()
  }
}

impl<const N: usize> PartialEq<[u8; N]> for FixedStr<N> {
  fn eq(&self, other: &[u8; N]) -> bool {
    self.as_bytes() == other
  }
}

impl<const N: usize> PartialEq<FixedStr<N>> for [u8; N] {
  fn eq(&self, other: &FixedStr<N>) -> bool {
    self == other.as_bytes()
  }
}

//******************************************************************************
//  Feature Implementations
//******************************************************************************

#[cfg(feature = "std")]
impl<const N: usize> PartialEq<Vec<u8>> for FixedStr<N> {
  fn eq(&self, other: &Vec<u8>) -> bool {
    self.as_bytes() == &**other
  }
}

#[cfg(feature = "std")]
impl<const N: usize> PartialEq<FixedStr<N>> for Vec<u8> {
  fn eq(&self, other: &FixedStr<N>) -> bool {
    &**self == other.as_bytes()
  }
}

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