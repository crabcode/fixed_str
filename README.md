# fixed_string

[![Crates.io](https://img.shields.io/crates/v/fixed_string)](https://crates.io/crates/fixed_string) [![Docs.rs](https://docs.rs/fixed_string/badge.svg)](https://docs.rs/fixed_string) [![License](https://img.shields.io/crates/l/fixed_string)](LICENSE)

**fixed_string** is a Rust crate that provides fixed–capacity, null–padded UTF‑8 string types designed for performance–critical, memory–sensitive, or FFI contexts. With predictable layout and safe UTF‑8 handling, it’s ideal for embedded applications, binary serialization, and other environments where precise control over memory is paramount.

## Overview

`fixed_string` introduces a primary type, `FixedStr<N>`, which uses a `[u8; N]` array as its internal storage. Unused bytes are zero–padded and the first null byte (`\0`) serves as the string terminator. This design guarantees:
- **Fixed capacity:** Memory usage is predictable.
- **Null–terminated semantics:** Operations like display and string slicing respect the first `\0` as the end.
- **UTF‑8 safety:** Input strings are truncated at valid character boundaries, ensuring no partial code points.
- **No dynamic allocation:** Suitable for no_std environments.

## Key Features

- **Fixed Capacity & Layout:** Each string is backed by a fixed-size array, ensuring predictable, constant memory usage.
- **Null-Terminated Semantics:** The string’s visible content stops at the first `\0`, while preserving the underlying data.
- **Safe UTF‑8 Truncation:** Truncation respects UTF‑8 boundaries—if a character cannot fully fit, it won’t be partially included.
- **Const–Safe Construction:** Use `FixedStr::new_const` to create fixed strings in constant contexts (with the caveat that UTF‑8 isn’t revalidated).
- **Incremental Building:** The `FixedStrBuf` builder allows you to construct fixed strings piece–by–piece with proper boundary checks.
- **Serialization Support:** Integrates with Serde for human–readable serialization and binrw for binary formats.
- **no_std Compatible:** Works in embedded and bare–metal systems.
- **Standard Trait Implementations:** Implements traits like `Clone`, `Copy`, `Debug`, `Display`, `PartialEq`, and more.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
fixed_string = "0.2.1"
```

Or run:

```vim
cargo add fixed_string --feature serde
```

Optional feature flags include:
- **std** – Enables standard library–dependent conversions and formatting (enabled by default).
- **serde** – For Serde serialization/deserialization.
- **binrw** – For binary serialization support.
- **const_mut_refs** – Opt-in for `const_mut_refs` (enabled by default, disable for compatibility with rustc versions <1.83).

## Usage Examples

### Creating a Fixed String

```rust
use fixed_string::FixedStr;

fn main() {
    // Create a FixedStr with 10 bytes of storage.
    let s = FixedStr::<10>::new("Hello, world!");
    // Output will be "Hello, wor" due to safe UTF‑8 truncation.
    println!("{}", s);
}
```

### Using the Builder: `FixedStrBuf`

```rust
use fixed_string::{FixedStrBuf, FixedStr};

fn main() {
    let mut buf = FixedStrBuf::<12>::new();
    buf.try_push_str("Hello").unwrap();
    buf.try_push_char(' ').unwrap();
    buf.push_str_lossy("world! 👋");
    let fixed: FixedStr<12> = buf.into_fixed();
    println!("{}", fixed); // Likely prints "Hello world!"
}
```

### Converting & Viewing

- **as_str():** Returns the string up to the first null byte.
- **effective_bytes():** Returns the underlying bytes up to the first null byte.
- **into_string():** Converts the fixed string into an owned `String` (requires the `std` feature).

## API Overview

### Constructors
- `FixedStr::new(&str) -> Self`: Create a fixed string with proper UTF‑8 truncation.
- `FixedStr::new_const(&str) -> Self`: Const–fn construction (without runtime UTF‑8 validation).

### Modifiers
- `set(&mut self, &str)`: Replace the content (truncated at the first `\0`).
- `clear()`: Zero out the internal buffer.
- `truncate(len: usize)`: Truncate the visible portion to a specified length.

### Views & Conversions
- `as_str() -> &str`: View the string up to the null terminator.
- `try_as_str() -> Result<&str, FixedStrError>`: UTF‑8 checked view.
- `as_bytes() -> &[u8]`: Raw byte view of the entire buffer.
- `effective_bytes() -> &[u8]`: View of the bytes until the first `\0`.
- `into_string() -> String`: Convert into an owned `String` (requires `std`).
- `to_string_lossy() -> String`: Lossy conversion if needed.

## License

This project is dual–licensed under either the MIT license or the Apache License, Version 2.0. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) for details.

## Links

- [Crate on crates.io](https://crates.io/crates/fixed_string)
- [Documentation on docs.rs](https://docs.rs/fixed_string)
- [GitHub Repository](https://github.com/crabcode/fixed_string)
