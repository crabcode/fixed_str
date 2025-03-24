#[cfg(feature = "memchr")]
use memchr::memchr;

/// Enforces `FixedStr` capacity to be greater than zero.
///
/// # Panics
/// Panics if N == 0.
pub const fn panic_on_zero(n: usize) {
  assert!(n > 0, "FixedStr capacity N must be greater than zero");
}

/// Finds the first `\0` byte in an array.
///
/// Returns the index of the first `\0`, or the full length if none found.
#[cfg(not(feature = "memchr"))]
pub fn find_first_null(bytes: &[u8]) -> usize {
  bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len())
}

/// Finds the first `\0` byte in an array, using speed-optimized `memchr`.
///
/// Returns the index of the first `\0`, or the full length if none found.
#[cfg(feature = "memchr")]
pub fn find_first_null(bytes: &[u8]) -> usize {
  memchr(0, bytes).unwrap_or(bytes.len())
}

/// Finds the largest index (up to `max_len` and the effective end) such that
/// the slice `bytes[..index]` is valid UTF‑8. This implementation uses a binary
/// search instead of decrementing one byte at a time.
///
/// # Parameters
/// - `bytes`: The input byte slice.
/// - `max_len`: The maximum number of bytes to consider.
///
/// # Returns
/// The largest valid length.
pub fn find_valid_utf8_len(bytes: &[u8], max_len: usize) -> usize {
  // Only consider bytes up to the first null (if any)
  let effective = find_first_null(bytes);
  let upper = max_len.min(effective);
  // If the entire prefix is valid, we’re done.
  if core::str::from_utf8(&bytes[..upper]).is_ok() {
    return upper;
  }
  // Otherwise, use binary search on the interval [0, upper].
  let mut low = 0;
  let mut high = upper;
  while low < high {
    // Bias the middle upward so that low eventually reaches the maximum valid index.
    let mid = (low + high + 1) / 2;
    if core::str::from_utf8(&bytes[..mid]).is_ok() {
      low = mid;
    } else {
      high = mid - 1;
    }
  }
  low
}

/// Truncates a byte slice to a valid UTF‑8 string within a maximum length.
///
/// # Returns
/// A string slice with only valid UTF‑8 bytes.
pub fn truncate_utf8_lossy(bytes: &[u8], max_len: usize) -> &str {
  let valid_len = find_valid_utf8_len(bytes, max_len);
  // SAFETY: We have computed a length for which the slice is valid UTF‑8.
  unsafe { core::str::from_utf8_unchecked(&bytes[..valid_len]) }
}

/// (Existing const function for const contexts.)
pub const fn find_valid_boundary(bytes: &[u8], max_len: usize) -> usize {
  let mut i = 0;
  let mut last_valid = 0;
  while i < bytes.len() {
    let first = bytes[i];
    let width = if first & 0x80 == 0 {
      1
    } else if (first & 0xE0) == 0xC0 {
      2
    } else if (first & 0xF0) == 0xE0 {
      3
    } else if (first & 0xF8) == 0xF0 {
      4
    } else {
      break; // Invalid leading byte.
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

    if j < i + width {
      break;
    }

    if i + width > max_len {
      break;
    }

    last_valid = i + width;
    i += width;
  }
  last_valid
}
