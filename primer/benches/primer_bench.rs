use criterion::{criterion_group, criterion_main, Criterion};
use primer::Primer;

fn primer_bench(c: &mut Criterion) {
    c.bench_function("crunch_10k_primes", |b| {
        b.iter(|| {
            let mut primer = Primer::new();
            primer.crunch(10_000, |_, _| false);
        });
    });
    c.bench_function("crunch_100k_primes", |b| {
        b.iter(|| {
            let mut primer = Primer::new();
            primer.crunch(100_000, |_, _| false);
        });
    });
    c.bench_function("crunch_500k_primes", |b| {
        b.iter(|| {
            let mut primer = Primer::new();
            primer.crunch(500_000, |_, _| false);
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = primer_bench
);
criterion_main!(benches);
