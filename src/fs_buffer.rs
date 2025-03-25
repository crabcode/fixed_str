// fixed_str/src/fs_buffer.rs

use super::*;

/// A builder for incrementally constructing a `FixedStr` with a fixed capacity.
/// It maintains an internal byte buffer and tracks the number of bytes currently written (the effective length).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FixedStrBuf<const N: usize> {
    pub(super) buffer: [u8; N],
    /// The number of bytes currently stored (i.e. the effective length).
    pub(super) len: usize,
}

impl<const N: usize> FixedStrBuf<N> {
    /// Returns the total capacity of the buffer.
    pub const fn capacity(&self) -> usize {
        N
    }
    /// Returns the number of bytes remaining in the buffer.
    pub fn remaining(&self) -> usize {
        N - self.len
    }
    /// Returns the number of bytes currently written to the buffer.
    pub fn len(&self) -> usize {
        self.len
    }
    /// Returns `true` if the buffer is empty (i.e. no bytes have been written).
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Creates a new, empty `FixedStrBuf`.
    ///
    /// # Panics
    /// Panics if `N == 0`. Zero‑length strings are not supported.
    pub const fn new() -> Self {
        panic_on_zero(N);
        Self {
            buffer: [0u8; N],
            len: 0,
        }
    }

    /// Attempts to interpret the current effective bytes (up to the first null) as a valid UTF‑8 string.
    ///
    /// Returns an error if the effective content is not valid UTF‑8.
    pub fn try_as_str(&self) -> Result<&str, FixedStrError> {
        core::str::from_utf8(self.effective_bytes()).map_err(|_| FixedStrError::InvalidUtf8)
    }

    /// Attempts to append the entire input string to the buffer.
    ///
    /// The function considers the effective bytes of the input (up to its first null, if any).
    /// If the input’s byte-length exceeds the remaining capacity, no data is appended and an error is returned.
    pub fn try_push_str(&mut self, s: &str) -> Result<(), FixedStrError> {
        let bytes = s.effective_bytes();
        if bytes.len() > self.remaining() {
            return Err(FixedStrError::Overflow {
                available: self.remaining(),
                found: bytes.len(),
            });
        }
        self.buffer[self.len..self.len + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len();
        Ok(())
    }

    /// Attempts to append a single character to the buffer.
    ///
    /// The character is first encoded in UTF‑8. Returns an error if the resulting encoding does not fit in the remaining space.
    ///
    /// **Note:** If the UTF‑8 encoding of the character includes a null byte (`\0`),
    /// any subsequent bytes in the encoding will be ignored when finalizing or displaying the result.
    pub fn try_push_char(&mut self, c: char) -> Result<(), FixedStrError> {
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        self.try_push_str(s)
    }

    /// Appends as many complete UTF‑8 characters from the input string as possible.
    ///
    /// If the entire string fits into the remaining capacity, it returns `true`.
    /// Otherwise, it appends only the valid initial segment (up to the last complete character) and returns `false`.
    pub fn push_str_lossy(&mut self, s: &str) -> bool {
        let remaining = self.remaining();
        let valid = if s.len() > remaining {
            truncate_utf8_lossy(s.as_bytes(), remaining)
        } else {
            s
        };

        let bytes = valid.as_bytes();
        if !bytes.is_empty() {
            self.buffer[self.len..self.len + bytes.len()].copy_from_slice(bytes);
            self.len += bytes.len();
        }

        bytes.len() == s.len()
    }

    /// Finalizes the builder into a `FixedStr`.
    ///
    /// This method zero‑pads the unused portion of the buffer and creates a `FixedStr`
    /// from the internal byte array. If the written content contains a null byte (`\0`),
    /// the resulting string will terminate at that null, ignoring any bytes that follow.
    pub fn finalize(mut self) -> FixedStr<N> {
        self.buffer[self.len..N].fill(0);
        FixedStr::from_bytes(self.buffer)
    }

    /// Finalizes the builder into a `FixedStr` without UTF-8 boundary checks.
    ///
    /// # Warning
    /// Use with care—this may produce values that may cause conversions to panic or comparisons to fail.
    ///
    /// This method zero‑pads the unused portion of the buffer and creates a `FixedStr`
    /// from the internal byte array. If the written content contains a null byte (`\0`),
    /// the resulting string will terminate at that null, ignoring any bytes that follow.
    pub fn finalize_unsafe(mut self) -> FixedStr<N> {
        self.buffer[self.len..N].fill(0);
        FixedStr::from_bytes_unsafe(self.buffer)
    }

    /// Clears the builder, resetting its effective length to zero and zero‑filling the buffer.
    pub fn clear(&mut self) {
        self.buffer.fill(0);
        self.len = 0;
    }

    /// Truncates the effective content of the buffer to `new_len` bytes.
    ///
    /// If `new_len` is less than the current effective length, the method zeroes out the removed portion
    /// and updates the effective length accordingly. If `new_len` is greater than or equal to the current length,
    /// no changes are made.
    pub fn truncate(&mut self, new_len: usize) {
        if new_len < self.len {
            for i in new_len..self.len {
                self.buffer[i] = 0;
            }
            self.len = new_len;
        }
    }

    /// Converts the effective bytes of the buffer to a `String` in a lossy manner,
    /// replacing any invalid UTF‑8 sequences with the Unicode replacement character.
    #[cfg(feature = "std")]
    pub fn to_string_lossy(&self) -> String {
        String::from_utf8_lossy(self.effective_bytes()).into_owned()
    }
}

//******************************************************************************
//  Implementations
//******************************************************************************

impl<const N: usize> fmt::Display for FixedStrBuf<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = core::str::from_utf8(&self.buffer[..self.len]).unwrap_or("<invalid UTF-8>");
        write!(f, "{}", s)
    }
}

impl<const N: usize> fmt::Debug for FixedStrBuf<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match str::from_utf8(&self.buffer[..self.len]) {
            Ok(s) => write!(f, "FixedStrBuf<{}>({:?})", N, s),
            Err(_) => write!(
                f,
                "FixedStrBuf<{}>(<invalid UTF-8>) {:?}",
                N,
                fast_format_hex::<384>(&self.buffer, 16, Some(8))
            ),
        }
    }
}

impl<const N: usize> EffectiveBytes for FixedStrBuf<N> {
    /// Returns the effective bytes (up to the first null byte) from the internal buffer.
    fn effective_bytes(&self) -> &[u8] {
        self.buffer.effective_bytes()
    }
}

impl<const N: usize> AsRef<[u8]> for FixedStrBuf<N> {
    fn as_ref(&self) -> &[u8] {
        &self.buffer
    }
}

impl<const N: usize> Default for FixedStrBuf<N> {
    fn default() -> Self {
        Self {
            buffer: [0; N],
            len: 0,
        }
    }
}

impl<const N: usize> core::ops::Deref for FixedStrBuf<N> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

/// Creates a `FixedStrBuf` from a `FixedStr`.
///
/// The effective length of the builder is taken from the `FixedStr` (i.e. the number
/// of valid bytes up to the first null).
///
/// # Panics
/// Panics if `N == 0`. Zero‑length strings are not supported.
impl<const N: usize> From<FixedStr<N>> for FixedStrBuf<N> {
    fn from(fixed: FixedStr<N>) -> Self {
        Self {
            buffer: fixed.data,
            len: fixed.len(),
        }
    }
}

/// Attempts to create a `FixedStrBuf` from a byte slice using exact copy semantics.
///
/// The method uses `BufferCopyMode::Exact` and computes the effective length by finding
/// the first null byte in the copied buffer.
///
/// # Panics
/// Panics if `N == 0`. Zero‑length strings are not supported.
impl<const N: usize> core::convert::TryFrom<&[u8]> for FixedStrBuf<N> {
    type Error = FixedStrError;
    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        let buf = copy_into_buffer(slice, BufferCopyMode::Exact)?;
        let effective_len = find_first_null(&buf);
        Ok(Self {
            buffer: buf,
            len: effective_len,
        })
    }
}

impl<const N: usize> Hash for FixedStrBuf<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Only hash the effective bytes (up to the first null) to represent the visible string.
        self.effective_bytes().hash(state);
    }
}

impl<const N: usize> IntoIterator for FixedStrBuf<N> {
    type Item = u8;
    type IntoIter = core::array::IntoIter<u8, N>;

    fn into_iter(self) -> Self::IntoIter {
        core::array::IntoIter::into_iter(self.buffer.into_iter())
    }
}

impl<const N: usize> Ord for FixedStrBuf<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare only the effective bytes (up to the first null) of each builder.
        self.effective_bytes().cmp(other.effective_bytes())
    }
}

impl<const N: usize> PartialOrd for FixedStrBuf<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const N: usize> PartialEq<[u8]> for FixedStrBuf<N> {
    fn eq(&self, other: &[u8]) -> bool {
        self.effective_bytes() == other.effective_bytes()
    }
}

impl<const N: usize> PartialEq<FixedStrBuf<N>> for [u8] {
    fn eq(&self, other: &FixedStrBuf<N>) -> bool {
        self.effective_bytes() == other.effective_bytes()
    }
}

impl<const N: usize> PartialEq<&[u8]> for FixedStrBuf<N> {
    fn eq(&self, other: &&[u8]) -> bool {
        self.effective_bytes() == other.effective_bytes()
    }
}

impl<const N: usize> PartialEq<FixedStrBuf<N>> for &[u8] {
    fn eq(&self, other: &FixedStrBuf<N>) -> bool {
        self.effective_bytes() == other.effective_bytes()
    }
}

impl<const N: usize> PartialEq<[u8; N]> for FixedStrBuf<N> {
    fn eq(&self, other: &[u8; N]) -> bool {
        self.effective_bytes() == other.effective_bytes()
    }
}

impl<const N: usize> PartialEq<FixedStrBuf<N>> for [u8; N] {
    fn eq(&self, other: &FixedStrBuf<N>) -> bool {
        self.effective_bytes() == other.effective_bytes()
    }
}

impl<const N: usize> PartialEq<FixedStr<N>> for FixedStrBuf<N> {
    fn eq(&self, other: &FixedStr<N>) -> bool {
        self.effective_bytes() == other.effective_bytes()
    }
}

impl<const N: usize> PartialEq<FixedStrBuf<N>> for FixedStr<N> {
    fn eq(&self, other: &FixedStrBuf<N>) -> bool {
        self.effective_bytes() == other.effective_bytes()
    }
}

#[cfg(feature = "std")]
impl<const N: usize> PartialEq<Vec<u8>> for FixedStrBuf<N> {
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.effective_bytes() == other.effective_bytes()
    }
}

#[cfg(feature = "std")]
impl<const N: usize> PartialEq<FixedStrBuf<N>> for Vec<u8> {
    fn eq(&self, other: &FixedStrBuf<N>) -> bool {
        self.effective_bytes() == other.effective_bytes()
    }
}

//******************************************************************************
//  Tests
//******************************************************************************

#[cfg(test)]
mod buffer_tests {
    use super::*;

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
        let fixed: FixedStr<5> = buf.finalize();
        assert_eq!(fixed.as_str(), "Hello");
    }

    #[test]
    fn test_finalize_trailing_zeros() {
        let mut buf = FixedStrBuf::<10>::new();
        buf.try_push_str("Hi").unwrap();
        let fixed: FixedStr<10> = buf.finalize();
        // The effective string is "Hi" and the rest are zeros.
        assert_eq!(fixed.len(), 2);
        assert_eq!(fixed.as_str(), "Hi");
        assert_eq!(fixed.as_bytes()[2], 0);
    }

    #[test]
    fn test_fixed_str_buf_clear() {
        let mut buf = FixedStrBuf::<10>::new();
        buf.try_push_str("Hello").unwrap();
        assert_eq!(buf.len(), 5);

        buf.clear();
        assert_eq!(buf.len(), 0);
        assert_eq!(&buf, &[0u8; 10]);

        // Can reuse safely
        buf.try_push_str("Rust").unwrap();
        assert_eq!(buf.len(), 4);
        assert_eq!(&buf[..4], b"Rust");
    }

    #[test]
    fn test_fixed_str_buf_try_from_slice() {
        let input = b"Hello!";
        // FixedStrBuf uses BufferCopyMode::Exact in its TryFrom implementation.
        // Since the input is longer than the buffer capacity, this should error.
        let result = FixedStrBuf::<5>::try_from(&input[..]);
        assert!(result.is_err());
    }

    #[test]
    fn test_fixed_str_buf_ordering() {
        let mut buf1 = FixedStrBuf::<10>::new();
        let mut buf2 = FixedStrBuf::<10>::new();
        buf1.try_push_str("Apple").unwrap();
        buf2.try_push_str("Banana").unwrap();

        assert!(buf1 < buf2);

        let mut buf3 = FixedStrBuf::<10>::new();
        buf3.try_push_str("Apple").unwrap();
        assert_eq!(buf1, buf3);
    }

    #[test]
    fn test_truncate_reduces_length() {
        let mut buf = FixedStrBuf::<10>::new();
        // Fill the buffer with "HelloWorld" (10 bytes).
        buf.try_push_str("HelloWorld").unwrap();
        assert_eq!(buf.len(), 10);
        // Truncate to 5 bytes.
        buf.truncate(5);
        assert_eq!(buf.len(), 5);
        // Finalize the buffer and check that the effective string is "Hello".
        let fixed = buf.finalize();
        assert_eq!(fixed.as_str(), "Hello");
        // Also check that the truncated portion of the buffer is zero.
        for &b in &buf.as_ref()[5..] {
            assert_eq!(b, 0);
        }
    }

    #[test]
    fn test_truncate_no_effect_when_new_len_is_greater() {
        let mut buf = FixedStrBuf::<10>::new();
        buf.try_push_str("Hi").unwrap();
        assert_eq!(buf.len(), 2);
        // Truncating to a value greater than current length should do nothing.
        buf.truncate(5);
        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn test_from_fixedstr_effective_length() {
        // Create a FixedStr with capacity 10 from a string that doesn't fill it.
        let fixed = FixedStr::<10>::new("Hello");
        // Convert to a FixedStrBuf using the From<FixedStr> implementation.
        let buf: FixedStrBuf<10> = fixed.into();
        // The effective length should be 5 ("Hello"), not 10.
        assert_eq!(buf.len(), 5);
        assert_eq!(buf.effective_bytes(), b"Hello");
    }

    #[test]
    fn test_try_from_slice_effective_length() {
        // Create a byte slice with an embedded null.
        let bytes = b"Hello\0World";
        // Create a FixedStrBuf from the slice.
        let buf = FixedStrBuf::<11>::try_from(&bytes[..]).unwrap();
        // The effective length should stop at the null byte (index 5).
        assert_eq!(buf.len(), 5);
        assert_eq!(buf.effective_bytes(), b"Hello");
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_fixed_str_buf_into_iter() {
        let mut buf = FixedStrBuf::<5>::new();
        buf.try_push_str("Hey").unwrap();
        let bytes: Vec<u8> = buf.into_iter().collect();
        assert_eq!(bytes[..3], *b"Hey");
        assert_eq!(bytes[3..], [0u8; 2]);
    }
}
