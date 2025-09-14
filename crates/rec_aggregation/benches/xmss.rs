use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};

fn xmss_light(c: &mut Criterion) {
    let mut g = c.benchmark_group("xmss_light");
    g.sample_size(10)
        .measurement_time(Duration::from_secs(30))
        .warm_up_time(Duration::from_secs(10));

    let n: usize = 100;
    let log_lifetime: usize = 16;
    g.throughput(Throughput::Elements(n as u64));
    g.bench_with_input(BenchmarkId::new("xmss", "light"), &(), |b, _| {
        b.iter(|| black_box(rec_aggregation::bench_api::bench_xmss(n, log_lifetime)))
    });
    g.finish();
}

fn xmss_heavy(c: &mut Criterion) {
    let mut g = c.benchmark_group("xmss_heavy");
    g.sample_size(10)
        .measurement_time(Duration::from_secs(60))
        .warm_up_time(Duration::from_secs(10));

    let n: usize = 500;
    let log_lifetime: usize = 32;
    g.throughput(Throughput::Elements(n as u64));
    g.bench_with_input(BenchmarkId::new("xmss", "heavy"), &(), |b, _| {
        b.iter(|| black_box(rec_aggregation::bench_api::bench_xmss(n, log_lifetime)))
    });
    g.finish();
}

criterion_group!(benches, xmss_light, xmss_heavy);
criterion_main!(benches);
