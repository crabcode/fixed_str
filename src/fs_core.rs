// fixed_str/src/fs_core.rs

use super::*;

/// A fixed–length string with a constant size of `N` bytes.
///
/// Internally, the string is stored in a `[u8; N]` array.
/// Unused bytes are left as zeros. When converting to a `&str`,
/// the first `\0` byte is considered the end of the string.
///
/// # Examples
/// ```
/// use fixed_str::FixedStr;
///
/// let fs = FixedStr::<5>::new("Hello");
/// assert_eq!(fs.as_str(), "Hello");
///  ```
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct FixedStr<const N: usize> {
    pub(super) data: [u8; N],
}

/// A fixed–length string with a constant size of `N` bytes.
///
/// Internally, the string is stored in a `[u8; N]` array.
/// Unused bytes are left as zeros. When converting to a `&str`,
/// the first `\0` byte is considered the end of the string.
///
/// # Examples
/// ```
/// use fixed_str::FixedStr;
///
/// let fs = FixedStr::<5>::new("Hello");
/// assert_eq!(fs.as_str(), "Hello");
///  ```
impl<const N: usize> FixedStr<N> {
    /// Returns the maximum capacity of the `FixedStr`.
    pub const fn capacity(&self) -> usize {
        N
    }
    /// Returns true if the bytes up to the first zero form a valid UTF-8 string.
    pub fn is_valid(&self) -> bool {
        self.try_as_str().is_ok()
    }
    /// Returns the number of valid bytes up to the first zero byte.
    pub fn len(&self) -> usize {
        find_first_null(self)
    }
    /// Returns whether the effective string is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    //****************************************************************************
    //  Constructors
    //****************************************************************************

    /// Creates a new `FixedStr` from the given input string.
    ///
    /// The input is copied into a fixed–size buffer. If it's longer than `N`,
    /// it is truncated safely at the last valid UTF‑8 boundary. If shorter,
    /// the remaining bytes are filled with zeros.
    ///
    /// **Note:** If the input contains a null byte (`\0`), the string terminates there.
    /// Any content after the first null byte is ignored.
    ///
    /// # Examples
    /// ```
    /// use fixed_str::FixedStr;
    ///
    /// // "Hello" fits exactly in a buffer of 5 bytes.
    /// let fs = FixedStr::<5>::new("Hello");
    /// assert_eq!(fs.as_str(), "Hello");
    ///
    /// // "Hello, World!" is truncated safely to "Hello".
    /// let fs = FixedStr::<5>::new("Hello, World!");
    /// assert_eq!(fs.as_str(), "Hello");
    /// ```
    ///
    /// # Panics
    /// Panics if `N == 0`. Zero-length strings are not supported.
    pub fn new(input: &str) -> Self {
        let buf = copy_into_buffer(input.as_bytes(), BufferCopyMode::Truncate).unwrap();
        Self { data: buf }
    }

    /// Creates a new `FixedStr` at compile time, truncating at the last valid UTF-8 boundary.
    ///
    /// If the string contains `\0`, the rest will be truncated.
    ///
    /// If the string contains multibyte characters near the edge of the buffer,
    /// they will be omitted silently. If no valid boundary is found, the result may be empty.
    ///
    /// # Behavior
    ///
    /// Ensures truncation at a valid UTF‑8 boundary, but does **not report** if truncation occurred.
    ///
    /// Use [`FixedStr::new`] in runtime contexts for stricter handling.
    ///
    /// # Panics
    /// Panics if `N == 0`. Zero-length strings are not supported.
    pub const fn new_const(input: &str) -> Self {
        panic_on_zero(N);
        let bytes = input.as_bytes();
        let mut buf = [0u8; N];
        let mut i = 0;
        let len = find_valid_boundary(bytes, N);

        while i < N && i < len {
            buf[i] = bytes[i];
            i += 1;
        }

        Self { data: buf }
    }
    /// Creates a `FixedStr` from a byte slice.
    ///
    /// If the slice length is less than `N`, only the first `slice.len()` bytes are
    /// copied. If it's longer than `N`, only the first `N` bytes are used, truncated
    /// safely at a valid UTF-8 boundary.
    ///
    /// **Note:** If the slice contains a null byte (`\0`), the effective string will
    /// end there.
    ///
    /// # Panics
    /// Panics if `N == 0`. Zero-length strings are not supported.
    pub fn from_slice(input: &[u8]) -> Self {
        // BufferCopyMode::Truncate is guaranteed to be safe (including UTF-8 validity)
        Self {
            data: copy_into_buffer(input, BufferCopyMode::Truncate).unwrap(),
        }
    }

    /// Creates a `FixedStr` from a slice without validating UTF‑8.
    ///
    /// This stores all bytes up to capacity, even if the result is not valid UTF‑8.
    ///
    /// **Note:** Any null byte (`\0`) will terminate the string early when using
    /// `as_str()` or when comparing values.
    ///
    /// # Warning
    ///
    /// Use with care—may produce values that panic on `as_str()` or comparison.
    ///
    /// # Panics
    /// Panics if `N == 0`. Zero-length strings are not supported.
    pub fn from_slice_unsafe(slice: &[u8]) -> Self {
        // BufferCopyMode::Slice is guaranteed to be safe (NOT including UTF-8 validity)
        Self {
            data: copy_into_buffer(slice, BufferCopyMode::Slice).unwrap(),
        }
    }

    /// Constructs a `FixedStr` from an array of bytes.
    ///
    /// Interprets a full byte array as a UTF‑8 string, truncating only for invalid boundaries.
    ///
    /// **Note:** If the byte array contains a null byte (`\0`), it will terminate the string
    /// early when interpreted or displayed.
    ///
    /// # Panics
    /// Panics if `N == 0`. Zero-length strings are not supported.
    pub fn from_bytes(bytes: [u8; N]) -> Self {
        // BufferCopyMode::Slice is guaranteed to be safe (including UTF-8 validity)
        Self {
            data: copy_into_buffer(&bytes, BufferCopyMode::Truncate).unwrap(),
        }
    }

    /// Stores a byte array without UTF-8 validation.
    ///
    /// The bytes are used as-is. This may result in invalid UTF-8,
    /// which can cause `as_str()` to panic or `try_as_str()` to fail.
    ///
    /// **Note:** The first null byte (`\0`) still acts as a terminator when converting
    /// or comparing strings.
    ///
    /// # Warning
    ///
    /// Use with care—may produce values that panic on `as_str()` or comparison.
    ///
    /// # Panics
    /// Panics if `N == 0`. Zero-length strings are not supported.
    pub fn from_bytes_unsafe(bytes: [u8; N]) -> Self {
        // BufferCopyMode::Slice is guaranteed to be safe (NOT including UTF-8 validity)
        Self {
            data: copy_into_buffer(&bytes, BufferCopyMode::Slice).unwrap(),
        }
    }

    //****************************************************************************
    //  Modifiers
    //****************************************************************************

    /// Updates the `FixedStr` with a new value, replacing the current content.
    ///
    /// The input string is copied into the internal buffer. If the input is longer
    /// than N, an error is thrown. If the input is shorter than N, the remaining
    /// bytes are set to zero.
    ///
    /// **Warning:** if `input` contains `\0`, the rest will be truncated.
    ///
    /// # Panics
    /// Panics if `N == 0`. Zero-length strings are not supported.
    pub fn set(&mut self, input: &str) -> Result<(), FixedStrError> {
        self.data = copy_into_buffer(input.effective_bytes(), BufferCopyMode::Exact)?;
        Ok(())
    }

    /// Truncates overflowing bytes down to the last valid UTF-8 string.
    ///
    /// **Warning:** if `input` contains `\0`, the rest will be truncated.
    ///
    /// # Examples
    ///
    /// use fixed_str::FixedStr;
    ///
    /// let mut fs = FixedStr::<5>::new("Hello");
    /// fs.set_lossy("World!");
    /// // "World!" is truncated to "World" because the capacity is 5 bytes.
    /// assert_eq!(fs.as_str(), "World");
    ///
    /// # Panics
    /// Panics if `N == 0`. Zero-length strings are not supported.
    pub fn set_lossy(&mut self, input: &str) {
        // BufferCopyMode::Truncate is guaranteed to be safe (including UTF-8 validity)
        self.data = copy_into_buffer(input.effective_bytes(), BufferCopyMode::Truncate).unwrap();
    }

    /// Clears the `FixedStr`, setting all bytes to zero.
    pub fn clear(&mut self) {
        self.data = [0u8; N];
    }

    /// Truncates the fixed string to `new_len` bytes.
    ///
    /// If `new_len` is less than the current effective length, the effective string is cut
    /// off at `new_len` and all bytes from `new_len` to capacity are set to zero. If `new_len`
    /// is greater than or equal to the current effective length, this method does nothing.
    pub fn truncate(&mut self, new_len: usize) {
        let current = self.len();
        if new_len < current {
            self.data[new_len..N].fill(0);
        }
    }

    //****************************************************************************
    //  Accessors
    //****************************************************************************

    /// Returns the string slice representation.
    #[track_caller]
    pub fn as_str(&self) -> &str {
        truncate_utf8_lossy(self, N)
    }

    /// Attempts to interpret the stored bytes as a UTF‑8 string.
    ///
    /// Returns an error if the data up to the first zero byte is not valid UTF‑8.
    pub fn try_as_str(&self) -> Result<&str, FixedStrError> {
        str::from_utf8(self.effective_bytes()).map_err(|_| FixedStrError::InvalidUtf8)
    }

    /// Returns the raw bytes stored in the `FixedStr`.
    pub const fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    #[cfg(feature = "const_mut_refs")]
    /// Returns the raw bytes stored in the `FixedStr` as `mut`
    pub const fn as_mut_bytes(&mut self) -> &mut [u8] {
        &mut self.data
    }

    #[cfg(not(feature = "const_mut_refs"))]
    /// Returns the raw bytes stored in the `FixedStr` as `mut`
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Returns an iterator that goes through the full byte
    /// array instead of terminating at the first `\0`.
    pub fn byte_iter(&self) -> impl Iterator<Item = u8> + '_ {
        self.data.iter().copied()
    }

    //****************************************************************************
    //  std Functions
    //****************************************************************************

    /// Converts the `FixedStr` to an owned String.
    #[cfg(feature = "std")]
    pub fn into_string(self) -> String {
        self.as_str().to_string()
    }

    /// Attempts to convert the `FixedStr` to an owned String.
    #[cfg(feature = "std")]
    pub fn try_into_string(self) -> Result<String, FixedStrError> {
        self.try_as_str().map(str::to_string)
    }

    /// Converts the `FixedStr` to an owned String in a lossy manner,
    /// replacing any invalid UTF‑8 sequences with the Unicode replacement character.
    #[cfg(feature = "std")]
    pub fn to_string_lossy(&self) -> String {
        String::from_utf8_lossy(&self.data[..self.len()]).into_owned()
    }
}
