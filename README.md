
# fixed_string

`fixed_string` is a Rust crate that provides fixed–length, null–padded UTF‑8 string types. These types are useful when you want precise, predictable layout (e.g. in serialization, interop, or embedded contexts) without losing safe UTF-8 handling.

The main type, `FixedStr<N>`, stores exactly N bytes internally. It zero-pads any unused space and treats the first `\0` byte as the end of the visible string for all human-readable operations (e.g. `Display`, `as_str`, etc).

---

## Features

- **Fixed Capacity:** Each `FixedStr<N>` instance uses a `[u8; N]` backing array. If the input is shorter than N, the unused bytes are zero-padded.
- **Null-Terminated Semantics:** The string is terminated at the first `\0` byte. Data after that is preserved in memory but excluded from most string views.
- **Safe UTF‑8 Truncation:** Input strings are never split mid–character. If too long, they are truncated to the last valid boundary.
- **Const Construction:** `new_const` allows fixed strings to be defined in `const` contexts (skipping UTF‑8 validation).
- **Incremental Builder:** `FixedStrBuf<N>` allows you to build fixed strings piece-by-piece with boundary checks and lossy support.
- **Modifiable In-Place:** Methods like `clear`, `set`, and `truncate` update the content without allocations.
- **Binary & Text Format Support:**
  - **Serde:** Serialize as human-readable strings or raw bytes.
  - **Binrw:** Deserialize directly from fixed-size byte arrays.
- **Standard Trait Support:** Implements `Clone`, `Copy`, `Debug`, `Display`, `Hash`, `PartialEq`, `Ord`, and `From<&str>`, etc.
- **No-Std Friendly:** Designed to work in embedded environments and bare-metal systems.

---

## Usage

```rust
use fixed_string::FixedStr;

fn main() {
    // Create a FixedStr with exactly 10 bytes of space
    let mut s = FixedStr::<10>::new("Hello, world!");
    println!("{}", s); // → "Hello, wor" (UTF-8 safe truncation)

    // Update the content
    s.set("Rust\0Extra"); 
    println!("{}", s); // → "Rust" (everything after `\0` is ignored)

    // Convert to owned String (requires std)
    let owned: String = s.into_string();
    println!("Owned: {}", owned);
}
```

---

## `FixedStrBuf` Builder

```rust
use fixed_string::{FixedStrBuf, FixedStr};

fn main() {
    let mut builder = FixedStrBuf::<12>::new();
    builder.try_push_str("Hello").unwrap();
    builder.try_push_char(' ').unwrap();
    builder.push_str_lossy("world! 👋");
    let fixed: FixedStr<12> = builder.into_fixed();
    println!("{}", fixed); // likely: "Hello world!"
}
```

### Builder Highlights
- `try_push_str` / `try_push_char`: Append only if fully valid
- `push_str_lossy`: Append as much as fits, stopping at valid UTF-8 boundaries
- `into_fixed`: Finalizes into `FixedStr<N>`
- `clear`: Reuse without reallocating

---

## API Overview

### Constructors
- `FixedStr::new(&str) -> Self`: Safely constructs with padding and truncation
- `FixedStr::new_const(&str) -> Self`: `const` fn version

### Modifiers
- `set(&mut self, &str)`: Replace content (truncates on `\0`)
- `clear()`: Zero the internal buffer
- `truncate(len: usize)`: Truncate to visible length

### Views
- `as_str() -> &str`: Returns string up to first `\0`
- `try_as_str() -> Result<&str, FixedStrError>`: UTF-8 checked
- `as_bytes() -> &[u8]`: Full raw byte view
- `effective_bytes() -> &[u8]`: View up to first `\0`
- `len() / capacity()`: Logical vs physical size

### Conversion (requires `std`)
- `into_string() -> String`
- `to_string_lossy() -> String`

### Extras
- `as_hex()` and `as_hex_dump()`: Debug-friendly byte views

---

## Feature Flags

- `std` – Enables conversions and formatting features requiring std
- `binrw` – Adds binary serialization via [`binrw`](https://docs.rs/binrw)
- `serde` – Enables human-readable and byte-level (via `serde_as_bytes`) serialization
- `const_mut_refs` – Opt-in workaround for some `const` fn limitations

---

## Important Design Notes

- `\0` is **treated as a terminator**. Content after it is ignored in most APIs.
- `N` must be greater than 0. Supplying zero will panic at runtime.
- `new_const()` does **not** validate UTF-8 boundaries. Use with care.
- Slices are always padded or truncated to exactly `N` bytes.

---

## Example: Serde as Bytes

```rust
#[derive(Serialize, Deserialize)]
struct Header {
    #[serde(with = "fixed_string::features::serde_as_bytes")]
    name: FixedStr<16>,
}
```

---

## License

Licensed under either:
- MIT license ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

---

## Links

- [Crate on crates.io](https://crates.io/crates/fixed_string)
- [Docs.rs](https://docs.rs/fixed_string)
- [GitHub Repository](https://github.com/crabcode/fixed_string)
