use criterion::{criterion_group, criterion_main, Criterion};
use primer::Primer;
use std::cell::Cell;

fn primer_bench(c: &mut Criterion) {
    c.bench_function("crunch_3m_primes", |b| {
        b.iter(|| {
            let mut primer = Primer::new();
            let prime_count = Cell::new(0usize);
            primer.crunch(
                || prime_count.get() < 3_000_000,
                |_, _| {
                    prime_count.set(prime_count.get() + 1);
                },
            );
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = primer_bench
);
criterion_main!(benches);
