use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use whir_p3::whir::config::{FoldingFactor, SecurityAssumption};

// Import the example proving function from the binary crate's source.
#[path = "../src/examples/prove_poseidon2.rs"]
mod prove_poseidon2_mod;
use prove_poseidon2_mod::prove_poseidon2;

fn bench_poseidon2_light(c: &mut Criterion) {
    let mut g = c.benchmark_group("poseidon2_light");
    g.sample_size(10)
        .measurement_time(Duration::from_secs(30))
        .warm_up_time(Duration::from_secs(10));

    let (l16, l24, skips, fold, inv) = (
        13usize,
        12usize,
        4usize,
        FoldingFactor::ConstantFromSecondRound(5, 3),
        2usize,
    );
    g.throughput(Throughput::Elements((1u64 << l16) + (1u64 << l24)));
    g.bench_with_input(BenchmarkId::new("poseidon2", "light"), &(), |b, _| {
        b.iter(|| {
            black_box(prove_poseidon2(
                l16,
                l24,
                skips,
                fold,
                inv,
                SecurityAssumption::CapacityBound,
                13,
                128,
                1,
                5,
                false,
            ))
        });
    });
    g.finish();
}

fn bench_poseidon2_heavy(c: &mut Criterion) {
    let mut g = c.benchmark_group("poseidon2_heavy");
    g.sample_size(10)
        .measurement_time(Duration::from_secs(60))
        .warm_up_time(Duration::from_secs(10));

    let (l16, l24, skips, fold, inv) = (
        17usize,
        17usize,
        4usize,
        FoldingFactor::ConstantFromSecondRound(7, 4),
        4usize,
    );
    g.throughput(Throughput::Elements((1u64 << l16) + (1u64 << l24)));
    g.bench_with_input(BenchmarkId::new("poseidon2", "heavy"), &(), |b, _| {
        b.iter(|| {
            black_box(prove_poseidon2(
                l16,
                l24,
                skips,
                fold,
                inv,
                SecurityAssumption::CapacityBound,
                16,
                128,
                5,
                3,
                false,
            ))
        });
    });
    g.finish();
}

criterion_group!(benches, bench_poseidon2_light, bench_poseidon2_heavy);
criterion_main!(benches);
