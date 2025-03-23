# Fixed String

Fixed String is a Rust crate that provides a fixed–length, null–padded UTF‑8 string type. The type, `FixedStr<N>`, stores exactly N bytes and is designed to handle both safe UTF‑8 truncation and constant contexts (via a separate constructor).

## Features

- **Fixed Capacity:** Each `FixedStr` instance uses a `[u8; N]` buffer. If the input string is shorter than N, the remaining bytes are zeros
- **Safe UTF‑8 Truncation:** The `new` method ensures that if the input is too long, it gets truncated at the last valid UTF‑8 boundary so that you never end up with invalid UTF‑8
- **Const Constructor:** Use `new_const` for compile–time string creation (be mindful that this version does not check for UTF‑8 boundaries)
- **Incremental Building:** `FixedStrBuf` allows you to construct your fixed-length strings piece-by-piece without allocations
- **Modifiable:** Methods like `clear` and `set` let you update your fixed string in place
- **Standard Trait Implementations:** Implements common traits like `Clone`, `Copy`, `Debug`, `Display`, `Hash`, `PartialOrd`, and `Ord`. It also supports convenient conversions from and to &str and String
- **no_std Compatible:** Works in environments without the standard library
- **Optional Binary Serialization:** With the `binrw` feature enabled, you can easily serialize and deserialize your fixed strings

## Installation

Install using Cargo:

```
cargo add fixed_string
```

Or add this to your `Cargo.toml`:

```
[dependencies]
fixed_string = "0.2"

[dependencies.fixed_string]
version = "0.2"
features = ["binrw"]       # enable binrw support
default-features = false   # disable std
```

## Usage

Here’s a quick example to show how you can create and use a `FixedStr`:

```
use fixed_string::FixedStr;

fn main() {
  // Create a FixedStr with a capacity of 8 bytes
  let mut fs = FixedStr::<10>::new("Hello, world!");
  // Since the input exceeds 10 bytes, it will be safely truncated
  println!("FixedStr: {}", fs); // Might print "Hello, wor"
  
  // Update the string content
  fs.set("Rust");
  println!("Updated FixedStr: {}", fs);
  
  // Convert to an owned String (requires the std feature)
  let owned: String = fs.into_string(); println!("Owned String: {}", owned);
}
```

### Incremental Building with FixedStrBuf

For scenarios where you need to build the string incrementally (e.g., parsing data or assembling pieces in an embedded environment), you can use the `FixedStrBuf` builder:

```
use fixed_string::{FixedStrBuf, FixedStr};

fn main() {
    // Create a builder for a fixed string with a capacity of 12 bytes
    let mut builder = FixedStrBuf::<12>::new();
    
    // Append a string slice. If it doesn't fully fit, try_push_str returns an error
    builder.try_push_str("Hello").expect("Push should succeed");
    
    // Append a character. This will succeed if the UTF‑8 encoded char fits
    builder.try_push_char(' ').expect("Push should succeed");
    
    // Use a lossy push to append as many complete characters as possible
    // Returns false if not all input could be appended
    let full_pushed = builder.push_str_lossy("world! Welcome!");
    println!("All pushed? {}", full_pushed);
    
    // Finalize the builder into a FixedStr
    let fixed: FixedStr<12> = builder.into_fixed();
    println!("Final FixedStr: {}", fixed);
}
```

The FixedStrBuf API provides:
- try_push_str and try_push_char: Methods that ensure no partial data is added
- push_str_lossy: Appends as many complete UTF‑8 characters as possible, useful when you prefer to salvage what fits
- clear: Resets the builder for reuse
- into_fixed: Finalizes and converts the builder into a FixedStr, zeroing out any unused portion

## API Overview

**Constructors**
  - `FixedStr::new(input: &str) -> Self`: Creates a new fixed–size string, safely truncating if the input is too long
  - `FixedStr::new_const(input: &str) -> Self`: A constant constructor (does **not** check for valid UTF‑8 boundaries!)

**Builder**
- `FixedStrBuf::new() -> Self`: Creates an empty builder
- `FixedStrBuf::try_push_str(&mut self, s: &str) -> Result<(), FixedStrError>`: Appends a string slice if it fully fits
- `FixedStrBuf::try_push_char(&mut self, c: char) -> Result<(), FixedStrError>`: Appends a single character
- `FixedStrBuf::push_str_lossy(&mut self, s: &str) -> bool`: Appends as many complete characters as possible
- `FixedStrBuf::clear(&mut self)`: Resets the builder
- `FixedStrBuf::into_fixed(self) -> FixedStr<N>`: Finalizes the builder, zero–padding any remaining capacity

**Modifiers**
  - `set(&mut self, input: &str)`: Replace the content of the fixed string with a new value
  - `clear(&mut self)`: Reset the string to an empty state (all bytes zero)
        
**Accessors**
  - `len() -> usize`: Returns the number of valid bytes (up to the first zero)
  - `capacity() -> usize`: Returns the total capacity
  - `as_str() -> &str`: Returns a string slice (panics if the effective bytes aren’t valid UTF‑8)
  - `try_as_str() -> Result<&str, FixedStrError>`: Attempts to convert the stored bytes to a &str, returning an error if invalid
  - `as_bytes() -> &[u8]`: Access the underlying bytes
        
**Conversions (std only)**
  - `into_string() -> String`: Converts the FixedStr into an owned String
  - `to_string_lossy() -> String`: Converts to a String, replacing invalid UTF‑8 sequences with the Unicode replacement character

**Extras**
  - `as_hex()` and `as_hex_dump()`: For visualizing the byte content in hexadecimal form

## License

This project is dual-licensed under either the MIT license or the Apache 2.0 license.
