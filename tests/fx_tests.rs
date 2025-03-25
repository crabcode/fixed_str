// fixed_str/tests/fs_tests.rs

#[cfg(test)]
mod fs_tests {
    use fixed_str::*;

    // Verifies that creating a FixedStr with an input that exactly fills the capacity works as expected.
    #[test]
    fn test_new_exact() {
        const N: usize = 5;
        let input = "Hello";
        let fixed = FixedStr::<N>::new(input);
        assert_eq!(fixed.len(), 5);
        assert_eq!(fixed.as_str(), "Hello");
    }

    // Checks that input shorter than the capacity is correctly stored and padded with zeros.
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

    // Ensures that FixedStr::new safely truncates input to avoid splitting multi-byte characters.
    #[test]
    fn test_new_truncation() {
        // "a😊b" is 6 bytes total: "a" (1 byte), "😊" (4 bytes), "b" (1 byte).
        // With N = 4, the function should truncate safely to "a".
        const N: usize = 4;
        let input = "a😊b";
        let fixed = FixedStr::<N>::new(input);
        assert_eq!(fixed.as_str(), "a");
    }

    // Validates that from_slice properly truncates a byte slice that cuts into a multi-byte character.
    #[test]
    fn test_from_slice_truncate_invalid_utf8() {
        const N: usize = 4;
        let input = "a😊b".as_bytes(); // 6 bytes total.
        let fixed = FixedStr::<N>::from_slice(input);
        // Only the valid prefix "a" should be preserved.
        assert_eq!(fixed.as_str(), "a");
    }

    // Checks that new_const produces a valid FixedStr at compile time for valid input.
    #[test]
    fn test_new_const_valid() {
        const N: usize = 5;
        const FIXED: FixedStr<N> = FixedStr::new_const("Hello");
        assert_eq!(FIXED.as_str(), "Hello");
    }

    // Verifies that new_const handles multi-byte characters safely by discarding incomplete characters.
    #[test]
    fn test_new_const_invalid_utf8() {
        // "é" is 2 bytes in UTF-8. With N = 1, new_const should discard the partial character.
        const N: usize = 1;
        let fixed = FixedStr::<N>::new_const("é");
        // Ensure that the resulting FixedStr is valid UTF-8.
        assert!(fixed.try_as_str().is_ok());
    }

    // Tests that from_slice_unsafe copies exactly N bytes from a slice.
    #[test]
    fn test_from_slice() {
        const N: usize = 5;
        let slice = b"Hello, world!";
        let fixed = FixedStr::<N>::from_slice_unsafe(slice);
        // Expect the first 5 bytes ("Hello") to be used.
        assert_eq!(fixed.as_str(), "Hello");
    }

    // Checks that try_from successfully constructs a FixedStr from a valid byte slice.
    #[test]
    fn test_try_from_slice_valid() {
        const N: usize = 5;
        let bytes = b"Hello";
        let fixed = FixedStr::<N>::try_from(&bytes[..]).unwrap();
        assert_eq!(fixed.as_str(), "Hello");
    }

    // Ensures that try_from panics (or returns an error) when the input exceeds capacity.
    #[test]
    #[should_panic]
    fn test_try_from_slice_overflow() {
        const N: usize = 5;
        let bytes = b"Hello!";
        let fixed = FixedStr::<N>::try_from(&bytes[..]).unwrap();
        assert_eq!(fixed.as_str(), "Hello");
    }

    // Verifies that from_bytes correctly interprets a null‑terminated byte array.
    #[test]
    fn test_from_bytes_valid() {
        let bytes = *b"Hi\0\0\0";
        let fixed = FixedStr::<5>::from_bytes(bytes);
        assert_eq!(fixed.as_str(), "Hi");
    }

    // Tests that from_bytes returns a valid FixedStr even if the underlying bytes contain invalid UTF-8,
    // as long as the effective string is valid.
    #[test]
    fn test_from_bytes_invalid_utf8() {
        let bytes = [0xFF, 0xFF, 0, 0, 0];
        let fixed = FixedStr::<5>::from_bytes(bytes);
        assert!(fixed.try_as_str().is_ok());
    }

    // Checks that the Default implementation creates a FixedStr with no effective content.
    #[test]
    fn test_default() {
        const N: usize = 5;
        let fixed: FixedStr<N> = Default::default();
        assert_eq!(fixed.len(), 0);
        assert_eq!(fixed.as_str(), "");
    }

    // Validates that Debug formatting for a valid FixedStr produces a quoted string.
    #[test]
    fn test_debug_format_valid() {
        const N: usize = 5;
        let fixed = FixedStr::<N>::new("Hello");
        let debug_str = format!("{:?}", fixed);
        assert_eq!(debug_str, "\"Hello\"");
    }

    // Checks that Display formatting returns the effective string.
    #[test]
    fn test_display() {
        const N: usize = 5;
        let fixed = FixedStr::<N>::new("Hello");
        let display_str = format!("{}", fixed);
        assert_eq!(display_str, "Hello");
    }

    // Tests that the IntoIterator implementation iterates over the effective bytes.
    #[test]
    fn test_into_iter() {
        const N: usize = 5;
        let fixed = FixedStr::<N>::new("Hello");
        let collected: Vec<u8> = fixed.into_iter().collect();
        assert_eq!(collected, b"Hello");
    }

    // Verifies equality comparisons between FixedStr and &str.
    #[test]
    fn test_equality() {
        const N: usize = 5;
        let fixed = FixedStr::<N>::new("Hello");
        assert_eq!(fixed, "Hello");
        assert_eq!("Hello", fixed);
    }

    // Ensures that truncation stops before a multi-byte character when capacity would split it.
    #[test]
    fn test_truncation_exact_boundary() {
        let smile = "😊"; // 4 bytes.
        let prefix = "ab"; // 2 bytes.
        let input = format!("{}{}", prefix, smile); // 6 bytes total.
        let fixed = FixedStr::<5>::new(&input);
        // Expect truncation before the emoji, resulting in "ab".
        assert_eq!(fixed.as_str(), "ab");
    }

    // Checks that FixedStr terminates at the first null byte in the underlying array.
    #[test]
    fn test_zero_termination() {
        let bytes = *b"Hello\0World";
        let fixed = FixedStr::<11>::from_slice_unsafe(&bytes);
        assert_eq!(fixed.len(), 5);
        assert_eq!(fixed.as_str(), "Hello");
    }

    // Verifies that clear() zeroes out the entire buffer.
    #[test]
    fn test_clear_zeroes_data() {
        let mut fixed = FixedStr::<5>::new("abc");
        fixed.clear();
        assert_eq!(fixed.as_bytes(), &[0, 0, 0, 0, 0]);
    }

    // Tests that the capacity method returns the correct buffer capacity.
    #[test]
    fn test_capacity() {
        let fixed = FixedStr::<8>::new("abc");
        assert_eq!(fixed.capacity(), 8);
    }

    // Verifies that from_bytes_unsafe returns the expected effective string.
    #[test]
    fn test_from_bytes_unsafe() {
        let bytes = *b"Raw!\0\0";
        let fixed = FixedStr::<6>::from_bytes_unsafe(bytes);
        assert_eq!(fixed.as_str(), "Raw!");
    }

    // Tests the set() and set_lossy() methods for updating the content.
    #[test]
    fn test_set_and_set_lossy() {
        let mut fixed = FixedStr::<5>::new("abc");
        fixed.set("xy").unwrap();
        assert_eq!(fixed.as_str(), "xy");

        fixed.set_lossy("hello world"); // Should truncate to "hello"
        assert_eq!(fixed.as_str(), "hello");
    }

    // Checks that is_valid() correctly identifies valid FixedStr instances.
    #[test]
    fn test_is_valid() {
        let valid = FixedStr::<5>::new("abc");
        assert!(valid.is_valid());

        let bytes = [0xff, 0xff, 0, 0, 0];
        let valid = FixedStr::<5>::from_bytes(bytes);
        assert!(valid.is_valid());
    }

    // Verifies that modifying the mutable byte slice reflects in the effective string.
    #[test]
    fn test_as_mut_bytes() {
        let mut fixed = FixedStr::<4>::new("rust");
        let bytes = fixed.as_mut_bytes();
        bytes[0] = b'R';
        assert_eq!(fixed.as_str(), "Rust");
    }

    // Tests the byte iterator, ensuring it returns effective bytes followed by trailing zeros.
    #[test]
    fn test_byte_iter() {
        let fixed = FixedStr::<5>::new("abc");
        let bytes: Vec<u8> = fixed.byte_iter().collect();
        assert_eq!(bytes[..3], *b"abc");
        assert_eq!(bytes[3..], [0u8; 2]);
    }

    // Checks that the truncate() method reduces the effective length and zeros out truncated bytes.
    #[test]
    fn test_truncate_reduces_effective_length() {
        let mut s = FixedStr::<10>::new("HelloWorld");
        assert_eq!(s.as_str(), "HelloWorld");
        s.truncate(5);
        assert_eq!(s.as_str(), "Hello");
        // Verify that bytes beyond the new effective length are zero.
        for &b in &s.as_bytes()[5..] {
            assert_eq!(b, 0);
        }
    }

    // Ensures that truncating to a value greater than the current effective length does nothing.
    #[test]
    fn test_truncate_no_effect_when_new_len_is_greater() {
        let mut s = FixedStr::<10>::new("Hi");
        assert_eq!(s.as_str(), "Hi");
        s.truncate(5);
        assert_eq!(s.as_str(), "Hi");
    }

    // Tests conversion of FixedStr into an owned String.
    #[cfg(feature = "std")]
    #[test]
    fn test_into_string() {
        let fixed = FixedStr::<5>::new("Hi");
        let s: String = fixed.into_string();
        assert_eq!(s, "Hi");
    }

    // Checks that to_string() on a FixedStr containing invalid UTF-8 produces a safe, lossy String.
    #[cfg(feature = "std")]
    #[test]
    fn test_to_string_invalid() {
        let invalid = FixedStr::<4>::from_bytes([b'H', 0xff, b'i', 0]);
        let safe = invalid.to_string();
        assert_eq!(safe, "H");
    }

    // Verifies that try_into_string() converts a FixedStr into a String when possible.
    #[cfg(feature = "std")]
    #[test]
    fn test_try_into_string() {
        let valid = FixedStr::<5>::new("Yes!");
        let string = valid.try_into_string().unwrap();
        assert_eq!(string, "Yes!");

        let also_valid = FixedStr::<5>::new("Still yes!");
        // new() safely truncates, so the output is "Still".
        assert_eq!(also_valid.try_into_string().unwrap(), "Still");
    }

    // Tests that a FixedStr can be safely created from a raw byte array via transmute.
    #[cfg(feature = "std")]
    #[test]
    fn test_transparency() {
        use std::mem::transmute;
        let arr: [u8; 5] = *b"Hey\0\0";
        let fixed: FixedStr<5> = unsafe { transmute(arr) };
        assert_eq!(fixed.as_str(), "Hey");
    }
}
