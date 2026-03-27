// Benchmarks for MetricsObserver.
// Requires: --features observer-metrics
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_metrics_observer_noop(_c: &mut Criterion) {
    // Placeholder — detailed benchmarks to be added in follow-up.
}

criterion_group!(benches, bench_metrics_observer_noop);
criterion_main!(benches);
