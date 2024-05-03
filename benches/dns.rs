use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rsgateway::dns::{dns::Packet, packet::PacketBuffer};

pub fn criterion_benchmark(c: &mut Criterion) {


    c.bench_function(
        "dns_parsing", 
        |b| {
            let mut pb = PacketBuffer::default();
            let input: &[u8] = &[
                0x98,0xf3,
                0x81,0x80,0x00,0x01,
                0x00,0x04,0x00,0x00,0x00,0x00,
                6, 'g' as u8, 'o' as u8, 'o' as u8, 'g' as u8, 'l' as u8, 'e' as u8, 
                3, 'c' as u8, 'o' as u8, 'm' as u8, 0x00,
                0, 2 as u8,
                0x00,0x01,
                0xc0,0x0c,0x00,0x02,
                0x00,0x01,0x00,0x00,
                0x21,0x25,0x00,0x06,
                3,'n' as u8, 's' as u8, '2' as u8,
                0xc0,0x0c,0xc0,0x0c,
                0x00,0x02,0x00,0x01,
                0x00,0x00,0x21,0x25,
                0x00,0x06,
                0x03,'n' as u8, 's' as u8, '4' as u8,
                0xc0,0x0c,0xc0,0x0c,
                0x00,0x02,0x00,0x01,
                0x00,0x00,0x21,0x25,
                0x00,0x06,
                0x03,'n' as u8, 's' as u8, '1' as u8,
                0xc0,0x0c,0xc0,0x0c,
                0x00,0x02,0x00,0x01,
                0x00,0x00,0x21,0x25,
                0x00,0x06,
                0x03,'n' as u8, 's' as u8, '2' as u8,
                0xc0,0x0c,
            ];
            pb.buf[0..input.len()].copy_from_slice(input);

            b.iter(|| {
                let _ = black_box(Packet::try_from(&mut pb));
            });
        }
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
