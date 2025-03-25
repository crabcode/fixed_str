// fixed_str/src/fs_error.rs

use super::*;

/// Custom error type for `FixedStr` conversions.
#[derive(Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FixedStrError {
    /// Returned when the length of the input exceeds capacity.
    ///
    /// This usually happens when converting from a byte slice or building a
    /// `FixedStrBuf` where the provided input exceeds capacity.
    ///
    /// - `remaining`: The remaining free bytes in the string.
    /// - `found`: The length of the string to be added.
    Overflow {
        /// The available space in bytes.
        available: usize,
        /// The length of the input.
        found: usize,
    },
    /// Returned when the byte content could not be parsed as valid UTF-8.
    InvalidUtf8,
}

impl fmt::Debug for FixedStrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow {
                available: remaining,
                found,
            } => {
                write!(f, "Overflow: available {}, found {}", remaining, found)
            }
            Self::InvalidUtf8 => write!(f, "InvalidUtf8"),
        }
    }
}

impl fmt::Display for FixedStrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow {
                available: remaining,
                found,
            } => {
                write!(
                    f,
                    "Overflow: tried to add {} bytes with only {} bytes available",
                    found, remaining
                )
            }
            Self::InvalidUtf8 => write!(f, "Invalid UTF-8"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for FixedStrError {}

//******************************************************************************
//  Tests
//******************************************************************************

#[cfg(all(test, feature = "std"))]
#[test]
fn test_error_display() {
    use super::*;
    let overflow_error = FixedStrError::Overflow {
        available: 2,
        found: 5,
    };
    assert_eq!(
        format!("{}", overflow_error),
        "Overflow: tried to add 5 bytes with only 2 bytes available"
    );
    let invalid_utf8_error = FixedStrError::InvalidUtf8;
    assert_eq!(format!("{}", invalid_utf8_error), "Invalid UTF-8");
}
