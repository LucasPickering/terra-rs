use criterion::{black_box, criterion_group, criterion_main, Criterion};
use terra::{Meter3, NoiseFnConfig, World, WorldConfig};

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("world-generation");
    group.sample_size(10);

    let config = WorldConfig::default();
    group.bench_function("world gen", |b| {
        b.iter(|| World::generate(black_box(config)))
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
