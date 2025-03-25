// fixed_str/src/string_helpers.rs

#[cfg(feature = "memchr")]
use memchr::memchr;

/// Specifies how bytes should be copied from a source slice into a fixed‑size buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferCopyMode {
    /// Requires that the source fits entirely into the buffer. If not, the function panics.
    Exact,
    /// Copies up to the capacity, discarding any extra bytes. UTF‑8 validity is not checked.
    Slice,
    /// Copies as many valid UTF‑8 bytes as possible, truncating the source safely if it exceeds the capacity.
    Truncate,
}

/// Ensures that the provided capacity is greater than zero.
///
/// # Panics
/// Panics if `n == 0`, since zero‑length strings are not supported.
pub const fn panic_on_zero(n: usize) {
    assert!(n > 0, "FixedStr capacity N must be greater than zero");
}

/// Finds the index of the first null byte (`\0`) in the given slice.
///
/// Returns the index of the first null byte, or the full length of the slice if no null is found.
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

/// Finds the largest index (up to `max_len` and not exceeding the first null) such that
/// the slice `bytes[..index]` is valid UTF‑8.
///
/// This implementation uses a binary search approach for efficiency.
///
/// # Parameters
/// - `bytes`: The input byte slice.
/// - `max_len`: The maximum number of bytes to consider.
///
/// # Returns
/// The largest index (≤ `max_len`) for which `bytes[..index]` is valid UTF‑8.
pub fn find_valid_utf8_len(bytes: &[u8], max_len: usize) -> usize {
    // Only consider bytes up to the first null (if any)
    let effective = find_first_null(bytes);
    let upper = max_len.min(effective);
    // If the entire prefix is valid UTF‑8, return it.
    if core::str::from_utf8(&bytes[..upper]).is_ok() {
        return upper;
    }
    // Otherwise, perform a binary search on the interval [0, upper] to find the largest valid prefix.
    let mut low = 0;
    let mut high = upper;
    while low < high {
        // Bias the midpoint upward to converge on the maximum valid index.
        let mid = (low + high + 1) / 2;
        if core::str::from_utf8(&bytes[..mid]).is_ok() {
            low = mid;
        } else {
            high = mid - 1;
        }
    }
    low
}

/// Truncates a byte slice to a valid UTF‑8 string within a specified maximum length.
///
/// # Returns
/// A string slice containing only valid UTF‑8 bytes from the start of `bytes` up to the maximum valid length.
pub fn truncate_utf8_lossy(bytes: &[u8], max_len: usize) -> &str {
    let valid_len = find_valid_utf8_len(bytes, max_len);
    // SAFETY: The computed `valid_len` guarantees that `bytes[..valid_len]` is valid UTF‑8.
    unsafe { core::str::from_utf8_unchecked(&bytes[..valid_len]) }
}

/// Finds the largest valid UTF‑8 boundary in the given byte slice within a constant context.
///
/// This function iterates through `bytes` up to `max_len` and returns the index immediately after the last complete UTF‑8 character.
///
/// # Parameters
/// - `bytes`: The input byte slice.
/// - `max_len`: The maximum number of bytes to consider.
///
/// # Returns
/// The index corresponding to the end of the last valid UTF‑8 character within `max_len`.
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
            break; // Invalid leading byte encountered.
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

/// Copies bytes from a source slice into a fixed‑size array of length `N`.
///
/// The behavior depends on the specified `mode`:
/// - `Exact`: Requires that the source fits entirely into the buffer; otherwise, returns an overflow error.
/// - `Slice`: Copies up to `N` bytes from the source, regardless of UTF‑8 validity.
/// - `Truncate`: Copies as many valid UTF‑8 bytes as possible (up to `N`), truncating the source safely.
///
/// # Panics
/// Panics if `N == 0` (zero‑length strings are not supported).
pub fn copy_into_buffer<const N: usize>(
    src: &[u8],
    mode: BufferCopyMode,
) -> Result<[u8; N], crate::FixedStrError> {
    panic_on_zero(N);
    let len = match mode {
        BufferCopyMode::Exact => {
            if src.len() > N {
                return Err(crate::FixedStrError::Overflow {
                    available: N,
                    found: src.len(),
                });
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

/// A constant lookup table that maps each `u8` value to its two-character uppercase hexadecimal representation.
const HEX_TABLE: [[u8; 2]; 256] = [
    *b"00", *b"01", *b"02", *b"03", *b"04", *b"05", *b"06", *b"07", *b"08", *b"09", *b"0A", *b"0B",
    *b"0C", *b"0D", *b"0E", *b"0F", *b"10", *b"11", *b"12", *b"13", *b"14", *b"15", *b"16", *b"17",
    *b"18", *b"19", *b"1A", *b"1B", *b"1C", *b"1D", *b"1E", *b"1F", *b"20", *b"21", *b"22", *b"23",
    *b"24", *b"25", *b"26", *b"27", *b"28", *b"29", *b"2A", *b"2B", *b"2C", *b"2D", *b"2E", *b"2F",
    *b"30", *b"31", *b"32", *b"33", *b"34", *b"35", *b"36", *b"37", *b"38", *b"39", *b"3A", *b"3B",
    *b"3C", *b"3D", *b"3E", *b"3F", *b"40", *b"41", *b"42", *b"43", *b"44", *b"45", *b"46", *b"47",
    *b"48", *b"49", *b"4A", *b"4B", *b"4C", *b"4D", *b"4E", *b"4F", *b"50", *b"51", *b"52", *b"53",
    *b"54", *b"55", *b"56", *b"57", *b"58", *b"59", *b"5A", *b"5B", *b"5C", *b"5D", *b"5E", *b"5F",
    *b"60", *b"61", *b"62", *b"63", *b"64", *b"65", *b"66", *b"67", *b"68", *b"69", *b"6A", *b"6B",
    *b"6C", *b"6D", *b"6E", *b"6F", *b"70", *b"71", *b"72", *b"73", *b"74", *b"75", *b"76", *b"77",
    *b"78", *b"79", *b"7A", *b"7B", *b"7C", *b"7D", *b"7E", *b"7F", *b"80", *b"81", *b"82", *b"83",
    *b"84", *b"85", *b"86", *b"87", *b"88", *b"89", *b"8A", *b"8B", *b"8C", *b"8D", *b"8E", *b"8F",
    *b"90", *b"91", *b"92", *b"93", *b"94", *b"95", *b"96", *b"97", *b"98", *b"99", *b"9A", *b"9B",
    *b"9C", *b"9D", *b"9E", *b"9F", *b"A0", *b"A1", *b"A2", *b"A3", *b"A4", *b"A5", *b"A6", *b"A7",
    *b"A8", *b"A9", *b"AA", *b"AB", *b"AC", *b"AD", *b"AE", *b"AF", *b"B0", *b"B1", *b"B2", *b"B3",
    *b"B4", *b"B5", *b"B6", *b"B7", *b"B8", *b"B9", *b"BA", *b"BB", *b"BC", *b"BD", *b"BE", *b"BF",
    *b"C0", *b"C1", *b"C2", *b"C3", *b"C4", *b"C5", *b"C6", *b"C7", *b"C8", *b"C9", *b"CA", *b"CB",
    *b"CC", *b"CD", *b"CE", *b"CF", *b"D0", *b"D1", *b"D2", *b"D3", *b"D4", *b"D5", *b"D6", *b"D7",
    *b"D8", *b"D9", *b"DA", *b"DB", *b"DC", *b"DD", *b"DE", *b"DF", *b"E0", *b"E1", *b"E2", *b"E3",
    *b"E4", *b"E5", *b"E6", *b"E7", *b"E8", *b"E9", *b"EA", *b"EB", *b"EC", *b"ED", *b"EE", *b"EF",
    *b"F0", *b"F1", *b"F2", *b"F3", *b"F4", *b"F5", *b"F6", *b"F7", *b"F8", *b"F9", *b"FA", *b"FB",
    *b"FC", *b"FD", *b"FE", *b"FF",
];

/// Formats the given byte slice as an uppercase hexadecimal string,
/// grouping bytes as specified and inserting spaces and newlines accordingly,
/// then returns a `FixedStr` containing the formatted output.
/// Any unused space in the output buffer is zero‑padded.
///
/// # Parameters
/// - `bytes`: The input byte slice to format.
/// - `group`: The number of bytes per group. A newline is inserted when a group is complete.
/// - `max_lines`: An optional limit to the number of output lines. If `None`, all groups are printed.
///
/// # Returns
/// A `FixedStr` containing the hex‑formatted representation of `bytes`.
///
/// # Panics
/// Panics if `group == 0`.
pub fn fast_format_hex<const N: usize>(
    bytes: &[u8],
    group: usize,
    max_lines: Option<usize>,
) -> crate::FixedStr<N> {
    assert!(group > 0, "Group number needs to be greater than zero");
    let mut buffer = [0u8; N];
    let mut pos = 0;
    let mut count_in_line = 0;
    let mut truncated = false;

    // Start with the first line.
    let mut line_count = 1;

    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 {
            if count_in_line == group {
                // If a line limit is set and reached, break out.
                if let Some(max) = max_lines {
                    if line_count >= max {
                        break;
                    }
                }
                if pos < N {
                    buffer[pos] = b'\n';
                    pos += 1;
                } else {
                    truncated = true;
                    break;
                }
                count_in_line = 0;
                line_count += 1;
            } else if pos < N {
                buffer[pos] = b' ';
                pos += 1;
            } else {
                truncated = true;
                break;
            }
        }

        // Write two hex digits for the current byte using the lookup table.
        if pos + 1 < N {
            let pair = HEX_TABLE[b as usize];
            buffer[pos] = pair[0];
            buffer[pos + 1] = pair[1];
            pos += 2;
        } else {
            truncated = true;
            break;
        }
        count_in_line += 1;
    }

    if truncated && pos >= 3 {
        pos = pos.saturating_sub(3);
        if pos + 3 <= N {
            buffer[pos] = b'.';
            buffer[pos + 1] = b'.';
            buffer[pos + 2] = b'.';
            pos += 3;
        }
    }

    buffer[pos..N].fill(0);

    // Safe due to controlled construction.
    crate::FixedStrBuf { buffer, len: pos }.finalize()
}

/// Outputs the full hexadecimal representation of `bytes` by invoking the provided callback
/// for each output byte.
///
/// # Parameters
/// - `bytes`: The input byte slice to format.
/// - `group`: The number of bytes per group. A newline is inserted after each complete group.
/// - `max_lines`: An optional limit to the number of output lines. If `None`, all lines are output.
/// - `write`: A callback function that receives each output byte (for example, to write to a console).
pub fn dump_as_hex(
    bytes: &[u8],
    group: usize,
    max_lines: Option<usize>,
    mut write: impl FnMut(u8),
) {
    assert!(group > 0, "Group number needs to be greater than zero");
    let mut count_in_line = 0;
    let mut line_count = 1;
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 {
            if count_in_line == group {
                if let Some(max) = max_lines {
                    if line_count >= max {
                        break;
                    }
                }
                write(b'\n');
                count_in_line = 0;
                line_count += 1;
            } else {
                write(b' ');
            }
        }
        let pair = HEX_TABLE[b as usize];
        write(pair[0]);
        write(pair[1]);
        count_in_line += 1;
    }
}

//******************************************************************************
//  Tests
//******************************************************************************

/// Test module for the string helper functions.
#[cfg(test)]
mod helper_tests {
    use super::*;

    #[test]
    fn test_truncate_utf8_lossy() {
        // Use a multi‑byte emoji such that max_len truncates before the complete character.
        let s = "d😊b"; // "d" (1 byte), "😊" (4 bytes), "b" (1 byte)
        let bytes = s.as_bytes();
        // With max_len = 4, only "d" is valid.
        let truncated = truncate_utf8_lossy(bytes, 4);
        assert_eq!(truncated, "d");
    }

    #[test]
    fn test_exact_success() {
        let src = b"Hello";
        // Since "Hello" is 5 bytes and the capacity is 10, this should succeed.
        let buf: [u8; 10] = copy_into_buffer::<10>(src, BufferCopyMode::Exact).unwrap();
        // The first 5 bytes should match; the remainder should be zero.
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
        // In Truncate mode, only the first 5 valid UTF‑8 bytes will be copied.
        let buf: [u8; 5] = copy_into_buffer::<5>(src, BufferCopyMode::Truncate).unwrap();
        assert_eq!(&buf, b"Hello");
    }

    #[test]
    fn test_fast_format_hex_full_output() {
        // Format the array [0x12, 0xAB, 0x00, 0xFF] with a group size of 2 and no line limit.
        let bytes = [0x12, 0xAB, 0x00, 0xFF];
        let hex = fast_format_hex::<32>(&bytes, 2, None);
        // Expected output: "12 AB\n00 FF"
        assert_eq!(hex, "12 AB\n00 FF");
    }

    #[test]
    fn test_fast_format_hex_line_limit() {
        // Use an array of 10 bytes of 0xFF; group them in groups of 3, and limit output to 2 lines.
        let bytes = [0xFF; 10];
        let hex = fast_format_hex::<64>(&bytes, 3, Some(2));
        // Expected output:
        // Line 1: "FF FF FF"
        // Line 2: "FF FF FF"
        // The formatter stops before processing further groups.
        assert_eq!(hex, "FF FF FF\nFF FF FF");
    }

    #[test]
    #[should_panic]
    fn test_panic_on_zero() {
        // This should panic because the capacity is zero.
        let _ = crate::FixedStr::<0>::new("test");
    }

    #[test]
    fn test_buffer_copy_mode_slice() {
        let input = b"Hello, world!";
        // In Slice mode, even if the input is longer than the buffer capacity,
        // it copies only the first N bytes.
        let buf = copy_into_buffer::<5>(input, BufferCopyMode::Slice).unwrap();
        assert_eq!(&buf, b"Hello");
    }

    #[test]
    #[should_panic]
    fn test_fast_format_hex_with_zero_group() {
        let _ = fast_format_hex::<32>(b"Test", 0, None);
    }

    #[cfg(feature = "std")]
    /// Helper function to collect output into a `Vec<u8>` for testing.
    fn collect_output(bytes: &[u8], group: usize, max_lines: Option<usize>) -> Vec<u8> {
        let mut output = Vec::new();
        dump_as_hex(bytes, group, max_lines, |b| output.push(b));
        output
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_debug_format_hex_full_output() {
        // Test with a small array and full output.
        let bytes = [0x12, 0xAB, 0x00, 0xFF];
        let result = collect_output(&bytes, 2, None);
        let s = std::str::from_utf8(&result).unwrap();
        // Expected: "12 AB\n00 FF"
        assert_eq!(s, "12 AB\n00 FF");
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_debug_format_hex_line_limit() {
        // Test with 10 bytes of 0xFF; group by 3 and limit to 2 lines.
        let bytes = [0xFF; 10];
        let result = collect_output(&bytes, 3, Some(2));
        let s = std::str::from_utf8(&result).unwrap();
        // Expected: "FF FF FF\nFF FF FF"
        assert_eq!(s, "FF FF FF\nFF FF FF");
    }
}
