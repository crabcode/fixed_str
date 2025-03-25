# Changelog

All notable changes to this project will be documented in this file.

## [0.9.1] – 2025-03-25

### Added
- **`truncate(len)` method:** Added `truncate()` to `FixedStr` and `FixedStrBuf` for shortening visible string length in-place.
- **`finalize_unsafe()` method:** for cases requiring direct construction without UTF‑8 validation.

### Changed
- **Updated crate-level and function-level docs** for clarity, accuracy, and consistency with actual behavior.
- **`FixedStrBuf::finalize()` now returns `FixedStr` directly** (was `Result`), reflecting that `finalize()` only produces valid UTF‑8.
- **Implemented `From<FixedStrBuf<N>> for FixedStr<N>`** for ergonomic conversion from builder to fixed string.
- Improved descriptions for `from_bytes`, `set_lossy`, and other modifiers to better reflect truncation and null-termination behavior.
    
### Removed
- `FixedStr::as_hex()` and `FixedStr::hex_dump()`: Removed from the core type to avoid side effects and formatting logic in core APIs.

  Hex formatting is still available via the `fast_format_hex()` and `dump_as_hex()` helper functions for manual use.

### Fixed
- **Corrected misleading note** on `FixedStr::new_const` to reflect that UTF‑8 is now always respected, even at compile time.
- Corrected the conversion implementations for `FixedStrBuf` (from `FixedStr` and via `TryFrom<&[u8]>`) so that the effective length (up to the first null byte) is used rather than the full array capacity. This ensures that builder operations such as appending and truncating behave correctly.
- Corrected docblocks and comments referring to outdated runtime validation behavior.


## [0.9.0] - 2025-03-25
### Added
- **FixedStr & FixedStrBuf:** Introduced a fixed–capacity, null–padded UTF‑8 string type and its incremental builder.
- **Safe UTF‑8 Truncation:** Implemented truncation that respects UTF‑8 boundaries to avoid cutting multi-byte characters.
- **Serialization Support:** Integrated with `binrw` for binary serialization and `serde` for human–readable serialization/deserialization.
- **no_std Compatibility:** The crate is fully compatible with no_std environments for embedded or memory-sensitive applications.
- **Helper Functions:** Added utility functions for boundary detection, hex formatting (`fast_format_hex` and `dump_as_hex`), and buffer copying modes.
- **Test Suite:** Extensive unit and integration tests covering edge cases such as input overflow, UTF‑8 validation, truncation correctness, and more.
- **Benchmark Tests:** Included benchmarks (using Criterion) to evaluate the performance of the hex formatting functions.
- **Documentation:** In-code documentation and README with usage examples, API overview, and installation instructions.

### Fixed
- No fixes as this is the initial release.
