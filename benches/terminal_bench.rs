use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

/// Benchmark terminal output processing throughput
fn bench_output_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("output_processing");

    for size in &[1024, 4096, 8192, 16384] {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let data = vec![b'A'; size];

            b.iter(|| {
                let mut buffer = Vec::with_capacity(size * 2);
                buffer.extend_from_slice(black_box(&data));
                buffer
            });
        });
    }

    group.finish();
}

/// Benchmark scrollback buffer management
fn bench_scrollback_management(c: &mut Criterion) {
    let mut group = c.benchmark_group("scrollback");

    group.bench_function("circular_buffer_10k", |b| {
        let max_size = 10000;
        let mut buffer = Vec::with_capacity(max_size);
        let line = b"This is a test line\n";

        b.iter(|| {
            buffer.extend_from_slice(black_box(line));
            if buffer.len() > max_size {
                let excess = buffer.len() - max_size;
                buffer.drain(..excess);
            }
        });
    });

    group.finish();
}

/// Benchmark memory allocation strategies
fn bench_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");

    group.bench_function("vec_with_capacity", |b| {
        b.iter(|| {
            let _v: Vec<u8> = Vec::with_capacity(black_box(1024));
        });
    });

    group.bench_function("vec_default", |b| {
        b.iter(|| {
            let _v: Vec<u8> = Vec::new();
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_output_processing,
    bench_scrollback_management,
    bench_memory_allocation
);
criterion_main!(benches);
