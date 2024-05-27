use criterion::{black_box, criterion_group, criterion_main, Criterion};
use encoding::percent;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function(
        "percent",
        |b| b.iter(||
            percent::escape(black_box(
                "padded%2Fwith%2Bvarious%25characters%3Fthat%3Dneed%24some%40escaping%2Bpaddedsowebreak%2F256bytes",
            ))
        )
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
