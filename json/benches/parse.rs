use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use json::parser::{parse, tokenize};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse_small.json", |b| {
        b.iter(|| {
            let input = File::open("benches/testdata/small.json").unwrap();
            let reader = BufReader::new(input);

            black_box({
                let tokens = tokenize(reader.lines()).unwrap();
                let mut iter = tokens.iter().peekable();
                let _ = parse(&mut iter).unwrap();
            })
        });
    });

    c.bench_function("parse_medium.json", |b| {
        b.iter(|| {
            let input = File::open("benches/testdata/medium.json").unwrap();
            let reader = BufReader::new(input);

            black_box({
                let tokens = tokenize(reader.lines()).unwrap();
                let mut iter = tokens.iter().peekable();
                let _ = parse(&mut iter).unwrap();
            })
        });
    });

    c.bench_function("parse_large.json", |b| {
        b.iter(|| {
            let input = File::open("benches/testdata/large.json").unwrap();
            let reader = BufReader::new(input);

            black_box({
                let tokens = tokenize(reader.lines()).unwrap();
                let mut iter = tokens.iter().peekable();
                let _ = parse(&mut iter).unwrap();
            })
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
