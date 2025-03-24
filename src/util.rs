// fixed_string/src/util.rs

use crate::EffectiveBytes;

/// Enforces `FixedStr` capacities of greater than zero.
///
/// # Parameters
/// 
/// - `n`: The N in question.
/// 
/// # Panics
/// 
/// Panics if N == 0
pub const fn panic_on_zero(n: usize) {
  assert!(n > 0, "FixedStr capacity N must be greater than zero");
}

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
pub fn compute_valid_len(input: &str, capacity: usize) -> usize {
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
  let mut len = max_len.min(bytes.effective_bytes().len());
  while len > 0 && core::str::from_utf8(&bytes[..len]).is_err() {
    len -= 1;
  }
  unsafe { core::str::from_utf8_unchecked(&bytes[..len]) }
}

/// Finds the first `\0` byte in an array.
///
/// # Parameters
/// 
/// - `bytes`: The input byte array.
///
/// # Returns
/// 
/// The index of the first `\0` byte, or length of the array if none found.
pub fn find_first_null(bytes: &[u8]) -> usize {
  bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len())
}


/// Locate the last valid UTF‑8 boundary <= `max_len`.
///
/// # Parameters
/// 
/// - `bytes`: The input byte array.
/// - `max_len`: The maximum number of bytes to consider from the beginning of `bytes`.
///
/// # Returns
/// 
/// The index of the last valid boundary.
pub const fn find_valid_boundary(bytes: &[u8], max_len: usize) -> usize {
  let mut i = 0;
  let mut last_valid = 0;
  while i < bytes.len() {
    let first = bytes[i];
    // Determine codepoint width from leading byte:
    let width = if first & 0x80 == 0 {
      1
    } else if (first & 0xE0) == 0xC0 {
      2
    } else if (first & 0xF0) == 0xE0 {
      3
    } else if (first & 0xF8) == 0xF0 {
      4
    } else {
      // Invalid leading byte; break
      break;
    };

    if i + width > bytes.len() {
      break;
    }

    let mut j = i + 1;

    while j < i + width {
      if (bytes[j] & 0xC0) != 0x80 {
        break;
      }
      j += 1;
    }

    // If we ended early, this is invalid:
    if j < i + width {
      break;
    }

    // If adding this codepoint would exceed `max_len`, stop.
    if i + width > max_len {
      break;
    }

    // Commit this codepoint, move on
    last_valid = i + width;
    i += width;
  }

  last_valid
}