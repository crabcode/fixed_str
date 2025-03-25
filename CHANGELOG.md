# Changelog

All notable changes to this project will be documented in this file.

## [0.9.1] – 2025-03-25

### Added
- **`truncate(len)` method:** Added `truncate()` to `FixedStr` and `FixedStrBuf` for shortening visible string length in-place.

### Changed
- **Updated crate-level and function-level docs** for clarity, accuracy, and consistency with actual behavior.
- **Corrected misleading note** on `FixedStr::new_const` to reflect that UTF‑8 is now always respected, even at compile time.
- Improved descriptions for `from_bytes`, `set_lossy`, and other modifiers to better reflect truncation and null-termination behavior.

### Fixed
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
