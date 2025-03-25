// fixed_str/tests/fs_tests.rs

#[cfg(test)]
mod fs_tests {
    use fixed_str::*;

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
    fn test_from_slice_truncate_invalid_utf8() {
        // Simulate a byte slice that cuts into the middle of a multi-byte char
        const N: usize = 4;
        let input = "a😊b".as_bytes(); // 6 bytes
        let fixed = FixedStr::<N>::from_slice(input); // Should only preserve "a"
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
        // "é" is two bytes in UTF‑8; with N = 1, new_const discard rather than copy only the first byte.
        const N: usize = 1;
        let fixed = FixedStr::<N>::new_const("é");
        assert!(fixed.try_as_str().is_ok());
    }

    #[test]
    fn test_from_slice() {
        const N: usize = 5;
        let slice = b"Hello, world!";
        let fixed = FixedStr::<N>::from_slice_unsafe(slice);
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
    #[should_panic]
    fn test_try_from_slice_overflow() {
        const N: usize = 5;
        let bytes = b"Hello!";
        let fixed = FixedStr::<N>::try_from(&bytes[..]).unwrap();
        assert_eq!(fixed.as_str(), "Hello");
    }

    #[test]
    fn test_from_bytes_valid() {
        let bytes = *b"Hi\0\0\0";
        let fixed = FixedStr::<5>::from_bytes(bytes);
        assert_eq!(fixed.as_str(), "Hi");
    }

    #[test]
    fn test_from_bytes_invalid_utf8() {
        let bytes = [0xFF, 0xFF, 0, 0, 0];
        let fixed = FixedStr::<5>::from_bytes(bytes);
        assert!(fixed.try_as_str().is_ok());
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
        assert_eq!("Hello", fixed);
    }

    #[test]
    fn test_truncation_exact_boundary() {
        let smile = "😊"; // 4 bytes
        let prefix = "ab"; // 2 bytes
        let input = format!("{}{}", prefix, smile); // 6 bytes
        let fixed = FixedStr::<5>::new(&input); // only 1 byte of emoji would fit
        assert_eq!(fixed.as_str(), "ab"); // must truncate *before* smile
    }

    #[test]
    fn test_zero_termination() {
        let bytes = *b"Hello\0World";
        let fixed = FixedStr::<11>::from_slice_unsafe(&bytes);
        assert_eq!(fixed.len(), 5); // terminates at first \0
        assert_eq!(fixed.as_str(), "Hello");
    }

    #[test]
    fn test_clear_zeroes_data() {
        let mut fixed = FixedStr::<5>::new("abc");
        fixed.clear();
        assert_eq!(fixed.as_bytes(), &[0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_capacity() {
        let fixed = FixedStr::<8>::new("abc");
        assert_eq!(fixed.capacity(), 8);
    }

    #[test]
    fn test_from_bytes_unsafe() {
        let bytes = *b"Raw!\0\0";
        let fixed = FixedStr::<6>::from_bytes_unsafe(bytes);
        assert_eq!(fixed.as_str(), "Raw!");
    }

    #[test]
    fn test_set_and_set_lossy() {
        let mut fixed = FixedStr::<5>::new("abc");
        fixed.set("xy").unwrap();
        assert_eq!(fixed.as_str(), "xy");

        fixed.set_lossy("hello world"); // should truncate
        assert_eq!(fixed.as_str(), "hello");
    }

    #[test]
    fn test_is_valid() {
        let valid = FixedStr::<5>::new("abc");
        assert!(valid.is_valid());

        let bytes = [0xff, 0xff, 0, 0, 0];
        let valid = FixedStr::<5>::from_bytes(bytes);
        assert!(valid.is_valid());
    }

    #[test]
    fn test_as_mut_bytes() {
        let mut fixed = FixedStr::<4>::new("rust");
        let bytes = fixed.as_mut_bytes();
        bytes[0] = b'R';
        assert_eq!(fixed.as_str(), "Rust");
    }

    #[test]
    fn test_byte_iter() {
        let fixed = FixedStr::<5>::new("abc");
        let bytes: Vec<u8> = fixed.byte_iter().collect();
        assert_eq!(bytes[..3], *b"abc");
        assert_eq!(bytes[3..], [0u8; 2]);
    }

    #[test]
    fn test_truncate_reduces_effective_length() {
        // Create a FixedStr with "HelloWorld" (10 bytes).
        let mut s = FixedStr::<10>::new("HelloWorld");
        assert_eq!(s.as_str(), "HelloWorld");
        // Truncate to 5 bytes.
        s.truncate(5);
        assert_eq!(s.as_str(), "Hello");
        // Verify that the remainder of the buffer is zeroed out.
        for &b in &s.as_bytes()[5..] {
            assert_eq!(b, 0);
        }
    }

    #[test]
    fn test_truncate_no_effect_when_new_len_is_greater() {
        // Create a FixedStr with "Hi" (2 bytes effective).
        let mut s = FixedStr::<10>::new("Hi");
        assert_eq!(s.as_str(), "Hi");
        // Attempt to "truncate" to a longer length.
        s.truncate(5);
        // The effective string remains unchanged.
        assert_eq!(s.as_str(), "Hi");
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_into_string() {
        let fixed = FixedStr::<5>::new("Hi");
        let s: String = fixed.into_string();
        assert_eq!(s, "Hi");
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_to_string_invalid() {
        let invalid = FixedStr::<4>::from_bytes([b'H', 0xff, b'i', 0]);
        let safe = invalid.to_string();
        assert_eq!(safe, "H");
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_try_into_string() {
        let valid = FixedStr::<5>::new("Yes!");
        let string = valid.try_into_string().unwrap();
        assert_eq!(string, "Yes!");

        let also_valid = FixedStr::<5>::new("Still yes!");
        // new truncates safely
        assert_eq!(also_valid.try_into_string().unwrap(), "Still");
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_transparency() {
        use std::mem::transmute;
        let arr: [u8; 5] = *b"Hey\0\0";
        let fixed: FixedStr<5> = unsafe { transmute(arr) };
        assert_eq!(fixed.as_str(), "Hey");
    }
}
