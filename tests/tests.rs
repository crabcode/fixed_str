#[cfg(test)]
mod tests {
    use fixed_string::*;
    use core::convert::TryFrom;

    #[test]
    fn test_new_exact() {
        const N: usize = 5;
        let input = "Hello";
        let fixed = FixedStr::<N>::new(input);
        assert_eq!(fixed.len(), 5);
        assert_eq!(fixed.as_str(), "Hello");
    }

    #[test]
    fn test_new_shorter() {
        const N: usize = 10;
        let input = "Hi";
        let fixed = FixedStr::<N>::new(input);
        assert_eq!(fixed.len(), 2);
        assert_eq!(fixed.as_str(), "Hi");
        let mut expected = [0u8; N];
        expected[..2].copy_from_slice(b"Hi");
        assert_eq!(fixed.as_bytes(), &expected);
    }

    #[test]
    fn test_new_truncation() {
        // "a😊b" consists of: "a" (1 byte), "😊" (4 bytes), "b" (1 byte) → 6 bytes total.
        // With N = 4, copying "a😊" would be unsafe since 4 bytes falls in the middle of the emoji.
        // So new() should truncate safely to only "a".
        const N: usize = 4;
        let input = "a😊b";
        let fixed = FixedStr::<N>::new(input);
        assert_eq!(fixed.as_str(), "a");
    }

    #[test]
    fn test_new_const_valid() {
        const N: usize = 5;
        const FIXED: FixedStr<N> = FixedStr::new_const("Hello");
        // Since "Hello" fits exactly, we get valid UTF‑8.
        assert_eq!(FIXED.as_str(), "Hello");
    }

    #[test]
    fn test_new_const_invalid_utf8() {
        // Using new_const with a multi-byte char and a capacity that forces a partial copy.
        // "é" is two bytes in UTF‑8; with N = 1, new_const copies only the first byte.
        const N: usize = 1;
        let fixed = FixedStr::<N>::new_const("é");
        assert!(fixed.try_as_str().is_err());
    }

    #[test]
    fn test_from_slice() {
        const N: usize = 5;
        let slice = b"Hello, world!";
        let fixed = FixedStr::<N>::from_slice(slice);
        // from_slice copies only N bytes, so we expect "Hello".
        assert_eq!(fixed.as_str(), "Hello");
    }

    #[test]
    fn test_try_from_slice_valid() {
        const N: usize = 5;
        let bytes = b"Hello";
        let fixed = FixedStr::<N>::try_from(&bytes[..]).unwrap();
        assert_eq!(fixed.as_str(), "Hello");
    }

    #[test]
    fn test_try_from_slice_invalid_length() {
        const N: usize = 5;
        let bytes = b"Hi";
        let result = FixedStr::<N>::try_from(&bytes[..]);
        assert!(result.is_err());
        if let Err(FixedStrError::WrongLength { expected, found }) = result {
            assert_eq!(expected, N);
            assert_eq!(found, 2);
        } else {
            panic!("Unexpected error type");
        }
    }

    #[test]
    fn test_default() {
        const N: usize = 5;
        let fixed: FixedStr<N> = Default::default();
        // default produces a string with all zeros so length is 0.
        assert_eq!(fixed.len(), 0);
        assert_eq!(fixed.as_str(), "");
    }

    #[test]
    fn test_debug_format_valid() {
        const N: usize = 5;
        let fixed = FixedStr::<N>::new("Hello");
        let debug_str = format!("{:?}", fixed);
        // Debug for valid UTF‑8 outputs a quoted string.
        assert_eq!(debug_str, "\"Hello\"");
    }

    #[test]
    fn test_debug_format_invalid() {
        // Create an invalid UTF‑8 FixedStr by using new_const that copies a partial multi-byte char.
        const N: usize = 1;
        let fixed = FixedStr::<N>::new_const("é");
        let debug_str = format!("{:?}", fixed);
        assert!(debug_str.contains("<invalid UTF-8>"));
    }

    #[test]
    fn test_display() {
        const N: usize = 5;
        let fixed = FixedStr::<N>::new("Hello");
        let display_str = format!("{}", fixed);
        assert_eq!(display_str, "Hello");
    }

    #[test]
    fn test_into_iter() {
        const N: usize = 5;
        let fixed = FixedStr::<N>::new("Hello");
        let collected: Vec<u8> = fixed.into_iter().collect();
        assert_eq!(collected, b"Hello");
    }

    #[test]
    fn test_equality() {
        const N: usize = 5;
        let fixed = FixedStr::<N>::new("Hello");
        assert_eq!(fixed, "Hello");
        let s = "Hello";
        assert_eq!(s, fixed);
    }

    #[test]
    fn test_error_display() {
        let wrong_length_error = FixedStrError::WrongLength { expected: 5, found: 2 };
        assert_eq!(
            format!("{}", wrong_length_error),
            "Wrong length: expected 5 bytes, found 2 bytes"
        );
        let invalid_utf8_error = FixedStrError::InvalidUtf8;
        assert_eq!(format!("{}", invalid_utf8_error), "Invalid UTF-8");
    }

    #[test]
    fn test_truncate_utf8_lossy() {
        // Use a multi-byte emoji and set max_len such that it would otherwise cut into the emoji.
        let s = "a😊b"; // "a" (1 byte), "😊" (4 bytes), "b" (1 byte)
        let bytes = s.as_bytes();
        // With max_len = 4, only "a" is valid.
        let truncated = FixedStr::<4>::truncate_utf8_lossy(bytes, 4);
        assert_eq!(truncated, "a");
    }

    #[test]
    fn test_as_hex() {
        #[cfg(feature = "std")]
        {
            const N: usize = 5;
            let fixed = FixedStr::<N>::new("Hello");
            let hex = fixed.as_hex();
            // "Hello" → [0x48, 0x65, 0x6C, 0x6C, 0x6F]
            let expected = "48 65 6C 6C 6F";
            assert_eq!(hex, expected);
        }
    }

    #[test]
    fn test_as_hex_dump() {
        #[cfg(feature = "std")]
        {
            // Create a FixedStr with capacity larger than the string.
            const N: usize = 16;
            let fixed = FixedStr::<N>::new("Hello");
            // With group size 8, first line contains "Hello" bytes plus trailing zeros.
            // "Hello" is 5 bytes: 48 65 6C 6C 6F, then 3 zeros → "00 00 00"
            // Second line is all zeros.
            let expected = "48 65 6C 6C 6F 00 00 00\n00 00 00 00 00 00 00 00";
            assert_eq!(fixed.as_hex_dump(), expected);
        }
    }

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
        let fixed: FixedStr<5> = buf.into_fixed();
        assert_eq!(fixed.as_str(), "Hello");
    }

    #[test]
    fn test_into_fixed_trailing_zeros() {
        let mut buf = FixedStrBuf::<10>::new();
        buf.try_push_str("Hi").unwrap();
        let fixed: FixedStr<10> = buf.into_fixed();
        // The effective string is "Hi" and the rest are zeros.
        assert_eq!(fixed.len(), 2);
        assert_eq!(fixed.as_str(), "Hi");
        for &b in &fixed.as_bytes()[2..] {
            assert_eq!(b, 0);
        }
    }
}