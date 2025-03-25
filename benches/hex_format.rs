// fixed_str/benches/hex_format.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fixed_str::fast_format_hex;

/// Generates dummy data: a vector of 1024 bytes cycling through 0 to 255.
fn generate_input() -> Vec<u8> {
    (0..1024).map(|i| (i % 256) as u8).collect()
}

/// Benchmarks the `fast_format_hex` function, which formats a byte slice into a hexadecimal string
/// using a no‑std–friendly fast formatter.
fn bench_fast_format_hex(c: &mut Criterion) {
    let bytes = generate_input();
    c.bench_function("fast_format_hex", |b| {
        b.iter(|| {
            let _ = fast_format_hex::<4096>(black_box(&bytes), 16, None);
        });
    });
}

/// Benchmarks the standard formatting approach using the `format!("{:02X}", b)` macro,
/// which allocates a new String for the output.
fn bench_std_format_hex(c: &mut Criterion) {
    let bytes = generate_input();
    c.bench_function("std_format_hex", |b| {
        b.iter(|| {
            let s: String = bytes
                .chunks(16)
                .map(|chunk| {
                    chunk
                        .iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .collect::<Vec<_>>()
                .join("\n");
            black_box(s);
        });
    });
}

criterion_group!(hex_benches, bench_fast_format_hex, bench_std_format_hex);
criterion_main!(hex_benches);
