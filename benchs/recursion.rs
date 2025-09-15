use std::hint::black_box;
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

fn bench_recursion_heavy(c: &mut Criterion) {
    let mut g = c.benchmark_group("recursion_heavy");
    g.sample_size(10)
        .measurement_time(Duration::from_secs(60))
        .warm_up_time(Duration::from_secs(10));

    let n: u64 = 1u64 << 25; // fixed variables = 25
    g.throughput(Throughput::Elements(n));
    g.bench_with_input(BenchmarkId::new("recursion", "heavy"), &(), |b, _| {
        b.iter(|| black_box(rec_aggregation::bench_recursion()))
    });
    g.finish();
}

criterion_group!(benches, bench_recursion_heavy);
criterion_main!(benches);
