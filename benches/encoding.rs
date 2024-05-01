use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rsgateway::encoding::{base64, percent};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function(
        "base64", 
        |b| b.iter(|| 
            base64::encode(black_box(
                "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nam dignissim consectetur enim, id dignissim libero. Duis ac rhoncus leo, at accumsan lacus. Suspendisse in feugiat enim. Donec molestie aliquet elementum. Aenean a porta justo. Duis interdum diam at consectetur porta. Pellentesque vel orci ut massa convallis fermentum. Vivamus et elit sem. Aliquam tristique posuere feugiat. Nulla non euismod ante. Ut aliquet at elit ut posuere. Nunc vestibulum nec sem et vulputate. Fusce ac rutrum ligula, at gravida orci. Phasellus sed commodo diam, vel varius magna. Vivamus vel neque mauris. Curabitur maximus iaculis nibh, ut sagittis justo aliquet nec. ".to_string()
            ))
        )
    );

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
