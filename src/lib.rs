// fixed_string/src/lib.rs

//! A fixed–length, null–padded UTF‑8 string type.
//!
//! `FixedStr<N>` stores exactly N bytes and pads or truncates as needed.
//!
//! # Note on UTF‑8 Safety
//! When using `new`, if the input is longer than N, it is safely truncated at the last valid UTF‑8 boundary.
//! The `new_const` method does not perform this check and should be used with care.

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
/// The core `FixedStr` library.
pub mod fixed_str;
/// A builder in `FixedStrBuf`.
pub mod fx_buf;
/// Custom error type for `FixedStr`.
pub mod fx_error;
/// Implementations for `FixedStr`.
pub mod fx_impl;
/// Optional integrations with `binrw` or `serde`.
pub mod serialize_ext;
/// Helper functions.
pub mod string_helpers;

pub use effective_bytes::{EffectiveBytes, EffectiveBytesIter};
pub use fixed_str::FixedStr;
pub use fx_buf::FixedStrBuf;
pub use fx_error::FixedStrError;
pub use string_helpers::{
    copy_into_buffer, dump_as_hex, fast_format_hex, find_first_null, find_valid_boundary,
    find_valid_utf8_len, panic_on_zero, truncate_utf8_lossy, BufferCopyMode,
};
