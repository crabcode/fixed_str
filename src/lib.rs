// fixed_string/src/lib.rs

//! A fixed–length, null–padded UTF‑8 string type.
//!
//! `FixedStr<N>` stores exactly N bytes and pads or truncates as needed.
//!
//! # Note on UTF‑8 Safety
//! When using `new`, if the input is longer than N, it is safely truncated at the last valid UTF‑8 boundary.
//! The `new_const` method does not perform this check and should be used with care.

//#![deny(missing_docs)]

#![cfg_attr(not(feature = "std"), no_std)]
use core::{fmt, str, borrow::Borrow, cmp::Ordering, hash::{Hash, Hasher}};

#[cfg(feature = "std")]
use std::string::String;

#[cfg(feature = "std")]
use std::vec::Vec;

pub mod fixed_str;
pub mod buffer;
pub mod error;
pub mod features;
pub mod impls;

pub use fixed_str::*;
pub use buffer::FixedStrBuf;
pub use error::FixedStrError;
