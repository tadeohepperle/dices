use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dices::*;

// cargo bench
pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("2d2000", |b| {
        b.iter(|| {
            let _ = Dice::build_from_string(black_box("2d200")).unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
