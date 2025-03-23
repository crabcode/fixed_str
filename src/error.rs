// fixed_string/src/error.rs

use super::*;

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
  /// Returned when the length of the input exceeds remaining length.
  ///
  /// - `remaining`: The remaining free bytes in the string.
  /// - `found`: The length of the string to be added.
  Overflow {
    /// The expected length in bytes.
    remaining: usize,
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
          Self::Overflow { remaining, found } => {
            write!(f, "Overflow: remaining {}, found {}", remaining, found)
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
          Self::Overflow { remaining, found } => {
            write!(f, "Overflow: tried to add {} bytes with only {} bytes remaining", found, remaining)
          },
          Self::InvalidUtf8 => write!(f, "Invalid UTF-8"),
          Self::Truncated => write!(f, "Truncated"),
      }
  }
}

#[cfg(feature = "std")]
impl std::error::Error for FixedStrError {}
