// fixed_str/src/fs_core.rs

use super::*;

/// A fixed–length string with a constant size of `N` bytes.
///
/// Internally, the string is stored in a `[u8; N]` array. Unused bytes are zeroed.
/// When converting to a `&str`, the first null byte (`\0`) is considered the end of the string.
///
/// **Note:** Zero-length strings (i.e. `N == 0`) are not supported and will cause a panic.
///
/// # Examples
/// ```
/// use fixed_str::FixedStr;
///
/// let fs = FixedStr::<5>::new("Hello");
/// assert_eq!(fs.as_str(), "Hello");
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct FixedStr<const N: usize> {
    pub(super) data: [u8; N],
}

impl<const N: usize> FixedStr<N> {
    /// Returns the maximum capacity of the `FixedStr`.
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Returns `true` if the effective bytes (up to the first null byte) form a valid UTF‑8 string.
    pub fn is_valid(&self) -> bool {
        self.try_as_str().is_ok()
    }

    /// Returns the number of valid bytes in the effective string (up to the first null byte).
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
    /// The input is copied into a fixed–size buffer. If the input is longer than `N` bytes,
    /// it is safely truncated at the last valid UTF‑8 boundary. If it is shorter than `N`,
    /// the remaining bytes are zero‑padded.
    ///
    /// **Note:** If the input contains a null byte (`\0`), the string terminates at that point,
    /// and any content after the first null byte is ignored.
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
    /// Panics if `N == 0`. Zero‑length strings are not supported.
    pub fn new(input: &str) -> Self {
        let buf = copy_into_buffer(input.as_bytes(), BufferCopyMode::Truncate).unwrap();
        Self { data: buf }
    }

    /// Creates a new `FixedStr` at compile time with safe truncation.
    ///
    /// The input is copied into the fixed buffer. If the input exceeds the capacity,
    /// it is silently truncated at the last valid UTF‑8 boundary. If the input contains
    /// a null byte (`\0`), the string terminates at that point and any subsequent data is ignored.
    ///
    /// **Note:** Truncation is performed without error reporting; if no valid boundary is found,
    /// the result may be empty. Use [`FixedStr::new`] in runtime contexts for stricter handling.
    ///
    /// # Panics
    /// Panics if `N == 0`. Zero‑length strings are not supported.
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
    /// If the slice is shorter than `N` bytes, all bytes are copied and the remaining
    /// bytes are zero‑padded. If the slice is longer than `N`, only the first `N` bytes are used,
    /// with truncation performed at a valid UTF‑8 boundary.
    ///
    /// **Note:** If the slice contains a null byte (`\0`), the effective string terminates at that position.
    ///
    /// # Panics
    /// Panics if `N == 0`. Zero‑length strings are not supported.
    pub fn from_slice(input: &[u8]) -> Self {
        Self {
            data: copy_into_buffer(input, BufferCopyMode::Truncate).unwrap(),
        }
    }

    /// Creates a `FixedStr` from a byte slice without validating UTF‑8.
    ///
    /// Copies all bytes up to capacity even if the result is not valid UTF‑8.
    /// Any null byte (`\0`) encountered will cause the effective string to terminate early
    /// when using `as_str()` or during comparisons.
    ///
    /// # Warning
    /// Use with care—this may produce values that may cause conversions to panic or comparisons to fail.
    ///
    /// # Panics
    /// Panics if `N == 0`. Zero‑length strings are not supported.
    pub fn from_slice_unsafe(slice: &[u8]) -> Self {
        Self {
            data: copy_into_buffer(slice, BufferCopyMode::Slice).unwrap(),
        }
    }

    /// Constructs a `FixedStr` from a full byte array.
    ///
    /// Interprets the entire array as a UTF‑8 string, truncating only at invalid boundaries.
    ///
    /// **Note:** If the array contains a null byte (`\0`), the string will terminate at that point.
    ///
    /// # Panics
    /// Panics if `N == 0`. Zero‑length strings are not supported.
    pub fn from_bytes(bytes: [u8; N]) -> Self {
        Self {
            data: copy_into_buffer(&bytes, BufferCopyMode::Truncate).unwrap(),
        }
    }

    /// Stores a byte array without validating UTF‑8.
    ///
    /// The bytes are used as‑is, which may result in an invalid UTF‑8 string.
    /// The first null byte (`\0`) still acts as a terminator in conversions and comparisons.
    ///
    /// # Warning
    /// Use with care—this may produce values that may cause conversions to panic or comparisons to fail.
    ///
    /// # Panics
    /// Panics if `N == 0`. Zero‑length strings are not supported.
    pub fn from_bytes_unsafe(bytes: [u8; N]) -> Self {
        Self {
            data: copy_into_buffer(&bytes, BufferCopyMode::Slice).unwrap(),
        }
    }

    //****************************************************************************
    //  Modifiers
    //****************************************************************************

    /// Updates the `FixedStr` with a new value, replacing the current content.
    ///
    /// The input string is copied into the internal buffer. If the input is longer than `N`
    /// bytes, an error is returned. If it is shorter, the remaining bytes are zero‑padded.
    ///
    /// **Warning:** If the input contains a null byte (`\0`), the string terminates at that point.
    ///
    /// # Panics
    /// Panics if `N == 0`. Zero‑length strings are not supported.
    pub fn set(&mut self, input: &str) -> Result<(), FixedStrError> {
        self.data = copy_into_buffer(input.effective_bytes(), BufferCopyMode::Exact)?;
        Ok(())
    }

    /// Updates the `FixedStr` with a new value, silently truncating any overflowing bytes
    /// at the last valid UTF‑8 boundary.
    ///
    /// **Warning:** If the input contains a null byte (`\0`), the string terminates at that point.
    ///
    /// # Examples
    /// ```
    /// use fixed_str::FixedStr;
    ///
    /// let mut fs = FixedStr::<5>::new("Hello");
    /// fs.set_lossy("World!");
    /// // "World!" is truncated to "World" because the capacity is 5 bytes.
    /// assert_eq!(fs.as_str(), "World");
    /// ```
    ///
    /// # Panics
    /// Panics if `N == 0`. Zero‑length strings are not supported.
    pub fn set_lossy(&mut self, input: &str) {
        self.data = copy_into_buffer(input.effective_bytes(), BufferCopyMode::Truncate).unwrap();
    }

    /// Clears the `FixedStr`, setting all bytes to zero.
    pub fn clear(&mut self) {
        self.data = [0u8; N];
    }

    /// Truncates the fixed string to `new_len` bytes.
    ///
    /// If `new_len` is less than the current effective length, the effective string is cut
    /// off at `new_len` and all bytes from `new_len` to capacity are set to zero.
    /// If `new_len` is greater than or equal to the current effective length, this method does nothing.
    pub fn truncate(&mut self, new_len: usize) {
        let current = self.len();
        if new_len < current {
            self.data[new_len..N].fill(0);
        }
    }

    //****************************************************************************
    //  Accessors
    //****************************************************************************

    /// Returns the string slice representation of the effective string.
    #[track_caller]
    pub fn as_str(&self) -> &str {
        truncate_utf8_lossy(self, N)
    }

    /// Attempts to interpret the stored effective bytes as a UTF‑8 string.
    ///
    /// Returns an error if the data up to the first null byte is not valid UTF‑8.
    pub fn try_as_str(&self) -> Result<&str, FixedStrError> {
        str::from_utf8(self.effective_bytes()).map_err(|_| FixedStrError::InvalidUtf8)
    }

    /// Returns the raw byte array stored in the `FixedStr`.
    pub const fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    #[cfg(feature = "const_mut_refs")]
    /// Returns the raw byte array stored in the `FixedStr` as mutable.
    pub const fn as_mut_bytes(&mut self) -> &mut [u8] {
        &mut self.data
    }

    #[cfg(not(feature = "const_mut_refs"))]
    /// Returns the raw byte array stored in the `FixedStr` as mutable.
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Returns an iterator over the entire internal byte array,
    /// including trailing zeroes beyond the effective string.
    pub fn byte_iter(&self) -> impl Iterator<Item = u8> + '_ {
        self.data.iter().copied()
    }

    //****************************************************************************
    //  std Functions
    //****************************************************************************

    /// Converts the `FixedStr` to an owned `String`.
    #[cfg(feature = "std")]
    pub fn into_string(self) -> String {
        self.as_str().to_string()
    }

    /// Attempts to convert the `FixedStr` to an owned `String`.
    #[cfg(feature = "std")]
    pub fn try_into_string(self) -> Result<String, FixedStrError> {
        self.try_as_str().map(str::to_string)
    }

    /// Converts the `FixedStr` to an owned `String` in a lossy manner,
    /// replacing any invalid UTF‑8 sequences with the Unicode replacement character.
    #[cfg(feature = "std")]
    pub fn to_string_lossy(&self) -> String {
        String::from_utf8_lossy(&self.data[..self.len()]).into_owned()
    }
}
