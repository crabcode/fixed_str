use super::*;

/// A trait that extracts the effective bytes from the type, i.e. up until the first `\0`.
pub trait EffectiveBytes {
  /// Returns the effective bytes up until the first `\0`.
  fn effective_bytes(&self) -> &[u8];
}

//******************************************************************************
//  Implementations
//******************************************************************************

impl<const N: usize> EffectiveBytes for FixedStr<N> {
  fn effective_bytes(&self) -> &[u8] {
      &self[..self.len()]
  }
}

impl<const N: usize> EffectiveBytes for &FixedStr<N> {
  fn effective_bytes(&self) -> &[u8] {
    (*self).effective_bytes()
  }
}

impl EffectiveBytes for [u8] {
  fn effective_bytes(&self) -> &[u8] {
    let end = find_first_null(self);
    &self[..end]
  }
}

impl<const N: usize> EffectiveBytes for [u8; N] {
  fn effective_bytes(&self) -> &[u8] {
    let end = find_first_null(self);
    &self[..end]
  }
}

impl EffectiveBytes for &str {
  fn effective_bytes(&self) -> &[u8] {
    self.as_bytes().effective_bytes()
  }
}

#[cfg(feature = "std")]
impl EffectiveBytes for String {
  fn effective_bytes(&self) -> &[u8] {
    self.as_bytes().effective_bytes()
  }
}

//******************************************************************************
//  Iterator
//******************************************************************************

/// Iterator that stops at the first `\0`, looking at only the effective string.
pub struct EffectiveBytesIter<const N: usize> {
  pub(super) data: [u8; N],
  pub(super) index: usize,
  pub(super) len: usize,
}

impl<const N: usize> Iterator for EffectiveBytesIter<N> {
  type Item = u8;

  fn next(&mut self) -> Option<Self::Item> {
    if self.index < self.len {
      let byte = self.data[self.index];
      self.index += 1;
      Some(byte)
    } else {
      None
    }
  }
}