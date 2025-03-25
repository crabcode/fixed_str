// fixed_str/src/lib.rs

//! A fixed–capacity, null–padded UTF‑8 string type for predictable layout and safe truncation.
//!
//! `FixedStr<N>` always stores exactly `N` bytes in a `[u8; N]` array.
//! Visible content ends at the first `\0`, forming the **effective string**—used for comparisons, hashing, and display.
//!
//! # Behavior
//! - **Shorter input** → null-padded to fill the buffer.
//! - **Longer input** → truncated safely at the last valid UTF-8 boundary.
//! - **`\0` in input** → string terminates there; remaining content is ignored.
//!
//! # Philosophy
//! - **String-first semantics:** Treats the content as a true string, not just a byte buffer.
//! - **Lossy by default:** Truncation favors UTF‑8 correctness over byte preservation.
//! - **Strict by choice:** `TryFrom`, `FixedStrBuf`, and unsafe methods offer stricter control.
//! - **Const-ready:** Use [`FixedStr::new_const`] for compile-time construction (with silent truncation).
//!
//! Also includes:
//! - [`FixedStrBuf<N>`]: a builder with boundary-aware methods like `try_push_str()` and `push_str_lossy()`
//! - Support for `serde`, `binrw`, and `no_std` environments.

#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
use core::{
    borrow::Borrow,
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    str,
};

#[cfg(feature = "std")]
use std::string::String;

#[cfg(feature = "std")]
use std::vec::Vec;

/// A trait to expose the string's non-zero bytes.
pub mod effective_bytes;
/// A builder in `FixedStrBuf`.
pub mod fs_buffer;
/// The core `FixedStr` library.
pub mod fs_core;
/// Custom error type for `FixedStr`.
pub mod fs_error;
/// Implementations for `FixedStr`.
pub mod fs_impl;
/// Optional integrations with `binrw` or `serde`.
pub mod serialize_ext;
/// Helper functions.
pub mod string_helpers;

pub use effective_bytes::{EffectiveBytes, EffectiveBytesIter};
pub use fs_buffer::FixedStrBuf;
pub use fs_core::FixedStr;
pub use fs_error::FixedStrError;
pub use string_helpers::{
    copy_into_buffer, dump_as_hex, fast_format_hex, find_first_null, find_valid_boundary,
    find_valid_utf8_len, panic_on_zero, truncate_utf8_lossy, BufferCopyMode,
};
