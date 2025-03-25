// fixed_string/benches/hex_format.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fixed_string::fast_format_hex;

/// Generate some dummy data.
fn generate_input() -> Vec<u8> {
    (0..1024).map(|i| (i % 256) as u8).collect()
}

/// The no-std fast formatter.
fn bench_fast_format_hex(c: &mut Criterion) {
    let bytes = generate_input();
    c.bench_function("fast_format_hex", |b| {
        b.iter(|| {
            let _ = fast_format_hex::<4096>(black_box(&bytes), 16, None);
        });
    });
}

/// The standard `format!("{:02X}", b)` with String allocation
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
