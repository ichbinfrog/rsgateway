use std::{
    fs::{self, File},
    io::{BufWriter, Read},
};

use archive::lz77::Buffer;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tempfile::tempfile;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("base64", |b| {
        let filename = "benches/testdata/les_miserables.txt";
        let mut input = File::open(filename).unwrap();
        let metadata = fs::metadata(filename).unwrap();
        let mut input_buffer: Vec<u8> = vec![0; metadata.len() as usize];
        input.read_exact(&mut input_buffer).unwrap();

        let mut buf: Buffer<'_> = Buffer::new(&input_buffer, 4095, 15);

        let f = tempfile().unwrap();
        let mut writer = BufWriter::new(f);

        b.iter(|| black_box(buf.compress(&mut writer).unwrap()));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
