use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn signature_matching_benchmark(c: &mut Criterion) {
    c.bench_function("signature_match", |b| {
        let input = "bash: pip: command not found";
        b.iter(|| black_box(input.contains("command not found")));
    });
}

criterion_group!(benches, signature_matching_benchmark);
criterion_main!(benches);
