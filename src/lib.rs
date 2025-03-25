// fixed_str/src/lib.rs

//! A fixed–capacity, null–padded UTF‑8 string type for predictable layout and safe truncation.
//!
//! `FixedStr<N>` always stores exactly `N` bytes in a `[u8; N]` array.
//! The visible content is defined as the bytes up to the first null byte (`\0`), which is used for
//! comparisons, hashing, and display.
//!
//! # Behavior
//! - **Shorter input:** Input that is shorter than `N` is null‑padded to fill the buffer.
//! - **Longer input:** Input that exceeds `N` is safely truncated at the last valid UTF‑8 boundary.
//! - **Null byte in input:** If a null byte is present in the input, the effective string ends there,
//!   and any subsequent bytes are ignored.
//!
//! # Philosophy
//! - **String-first semantics:** The type treats the content as a genuine string rather than merely a raw byte array.
//! - **Lossy by default:** Truncation prioritizes preserving valid UTF‑8 over preserving every byte.
//! - **Strict by choice:** Methods like `TryFrom`, the builder (`FixedStrBuf`), and unsafe functions provide stricter control when needed.
//! - **Const-ready:** Use [`FixedStr::new_const`] for compile-time construction, which performs silent truncation.
//!
//! Also included:
//! - [`FixedStrBuf<N>`]: A builder for incrementally constructing `FixedStr` values with boundary-aware methods such as `try_push_str()` and `push_str_lossy()`.
//! - Optional integrations for `serde`, `binrw`, and support for `no_std` environments.

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

/// Exposes the effective (non‑zero) bytes of a `FixedStr`.
pub mod effective_bytes;
/// Provides the builder type `FixedStrBuf` for constructing fixed‑capacity strings.
pub mod fs_buffer;
/// Contains the core implementation of the `FixedStr` type.
pub mod fs_core;
/// Defines custom error types for the `FixedStr` library.
pub mod fs_error;
/// Implements various trait implementations for `FixedStr`.
pub mod fs_impl;
/// Provides optional integrations for binary and serialization support (`binrw` and `serde`).
pub mod serialize_ext;
/// Contains helper functions for byte copying, UTF‑8 boundary detection, and hex formatting.
pub mod string_helpers;

pub use effective_bytes::{EffectiveBytes, EffectiveBytesIter};
pub use fs_buffer::FixedStrBuf;
pub use fs_core::FixedStr;
pub use fs_error::FixedStrError;
pub use string_helpers::{
    copy_into_buffer, dump_as_hex, fast_format_hex, find_first_null, find_valid_boundary,
    find_valid_utf8_len, panic_on_zero, truncate_utf8_lossy, BufferCopyMode,
};
