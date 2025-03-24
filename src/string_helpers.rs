// fixed_string/src/string_helpers.rs

#[cfg(feature = "memchr")]
use memchr::memchr;

/// An enum to select copy mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferCopyMode {
    /// Requires that the source fits entirely into the buffer.
    Exact,
    /// Copies up to the capacity, discarding any extra bytes.
    Slice,
    /// Copies up to the capacity, truncating safely.
    Truncate,
}

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
pub fn find_first_null(bytes: &[u8]) -> usize {
  #[cfg(not(feature = "memchr"))]
  {
    bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len())
  }
  #[cfg(feature = "memchr")]
  {
    memchr(0, bytes).unwrap_or(bytes.len())
  }
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

/// Finds the valid string boundaries with in const context.
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

/// Copies bytes from a source slice into a fixed-size buffer of length N.
/// Depending on `mode`, it will either error (Exact) or truncate (Truncate) if the source is too long.
/// 
/// # Panics
/// Panics if N == 0.
pub fn copy_into_buffer<const N: usize>(src: &[u8], mode: BufferCopyMode) -> Result<[u8; N], crate::FixedStrError> {
  panic_on_zero(N);
  let len = match mode {
      BufferCopyMode::Exact => {
          if src.len() > N {
              return Err(crate::FixedStrError::Overflow { available: N, found: src.len() });
          }
          src.len()
      }
      BufferCopyMode::Slice => src.len().min(N),
      BufferCopyMode::Truncate => find_valid_utf8_len(src, N),
  };
  let mut buf = [0u8; N];
  buf[..len].copy_from_slice(&src[..len]);
  Ok(buf)
}

//******************************************************************************
//  Tests
//******************************************************************************

/// Test module for `string_helpers`.
#[cfg(test)]
mod helper_tests {
  use super::*;

  #[test]
  fn test_truncate_utf8_lossy() {
    // Use a multi-byte emoji and set max_len such that it would otherwise cut into the emoji.
    let s = "d😊b"; // "a" (1 byte), "😊" (4 bytes), "b" (1 byte)
    let bytes = s.as_bytes();
    // With max_len = 4, only "d" is valid.
    let truncated = truncate_utf8_lossy(bytes, 4);
    assert_eq!(truncated, "d");
  }

  #[test]
  fn test_exact_success() {
      let src = b"Hello";
      // Since "Hello" is 5 bytes and the capacity is 10, this succeeds.
      let buf: [u8; 10] = copy_into_buffer::<10>(src, BufferCopyMode::Exact).unwrap();
      // The first 5 bytes match; the rest should be zero.
      assert_eq!(&buf[..5], src);
      assert_eq!(&buf[5..], &[0; 5]);
  }

  #[test]
  fn test_exact_overflow() {
      let src = b"Hello, world!";
      let res = copy_into_buffer::<5>(src, BufferCopyMode::Exact);
      assert!(res.is_err());
  }

  #[test]
  fn test_truncate() {
      let src = b"Hello, world!";
      // In Truncate mode, only the first 5 bytes will be copied.
      let buf: [u8; 5] = copy_into_buffer::<5>(src, BufferCopyMode::Truncate).unwrap();
      assert_eq!(&buf, b"Hello");
  }
}