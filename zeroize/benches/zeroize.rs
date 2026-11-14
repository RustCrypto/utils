//! `zeroize` benchmarks.

#![allow(missing_docs, reason = "benchmark")]

use core::hint::black_box;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use zeroize::Zeroize;

fn bench_slices(c: &mut Criterion) {
    let mut group = c.benchmark_group("slice/u8");
    for len in [16, 32, 63, 64, 65, 128, 4096, 65536] {
        let mut bytes = vec![0xa5u8; len];
        group.throughput(Throughput::Bytes(len as u64));
        group.bench_function(BenchmarkId::from_parameter(len), |b| {
            b.iter(|| black_box(bytes.as_mut_slice()).zeroize());
        });
    }
    group.finish();
}

criterion_group!(benches, bench_slices);
criterion_main!(benches);
