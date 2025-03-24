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
pub mod impls;
/// A builder in `FixedStrBuf`.
pub mod buffer;
/// A trait to expose the string's non-zero bytes.
pub mod effective;
/// Custom error type for `FixedStr` conversions.
pub mod error;
/// Feature implementations, like `binrw` or `serde`.
pub mod features;
/// Helper functions.
pub mod util;

pub use fixed_str::*;
pub use buffer::FixedStrBuf;
pub use effective::*;
pub use error::FixedStrError;
pub use util::*;