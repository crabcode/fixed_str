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
use core::{fmt, str, borrow::Borrow, cmp::Ordering, hash::{Hash, Hasher}};

#[cfg(feature = "std")]
use std::string::String;

#[cfg(feature = "std")]
use std::vec::Vec;

/// The core `FixedStr` library.
pub mod fixed_str;
/// Implementations for `FixedStr`.
pub mod fx_impl;
/// A builder in `FixedStrBuf`.
pub mod fx_buf;
/// Custom error type for `FixedStr`.
pub mod fx_error;
/// A trait to expose the string's non-zero bytes.
pub mod effective_bytes;
/// Optional integrations with `binrw` or `serde`.
pub mod serialize_ext;
/// Helper functions.
pub mod string_helpers;

pub use fixed_str::FixedStr;
pub use fx_buf::FixedStrBuf;
pub use fx_error::FixedStrError;
pub use effective_bytes::{EffectiveBytes, EffectiveBytesIter};
pub use string_helpers::{
  panic_on_zero,
  find_first_null,
  find_valid_utf8_len,
  find_valid_boundary,
  truncate_utf8_lossy
};