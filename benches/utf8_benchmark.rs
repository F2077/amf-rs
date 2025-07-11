use amf_rs::utf8::Utf8;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::borrow::Cow;

fn benchmark_data() -> (&'static str, String) {
    let short_str = "hello, world";
    let long_str = "a".repeat(1024);
    (short_str, long_str)
}

/// 基准测试序列化操作
fn bench_serialization(c: &mut Criterion) {
    let (short_str, long_str_owned) = benchmark_data();

    let utf8_short = Utf8::new_borrowed(short_str).unwrap();
    let utf8_long = Utf8::new(Cow::Owned(long_str_owned)).unwrap();

    let mut group = c.benchmark_group("Serialization");

    // 1. 测试 to_bytes (会分配 Vec)
    group.bench_function("to_bytes (short)", |b| b.iter(|| utf8_short.to_bytes()));

    group.bench_function("to_bytes (long)", |b| b.iter(|| utf8_long.to_bytes()));

    // 2. 测试 write_to (写入已分配的 buffer)
    let mut short_buffer = vec![0; utf8_short.bytes_size() as usize];
    group.bench_function("write_to (short)", |b| {
        b.iter(|| utf8_short.write_to(black_box(&mut short_buffer)))
    });

    let mut long_buffer = vec![0; utf8_long.bytes_size() as usize];
    group.bench_function("write_to (long)", |b| {
        b.iter(|| utf8_long.write_to(black_box(&mut long_buffer)))
    });

    group.finish();
}

/// 基准测试反序列化操作 (这是最重要的对比)
fn bench_deserialization(c: &mut Criterion) {
    let (short_str, long_str_owned) = benchmark_data();

    let bytes_short = Utf8::new_borrowed(short_str).unwrap().to_bytes();
    let bytes_long = Utf8::new(Cow::Owned(long_str_owned)).unwrap().to_bytes();

    let mut group = c.benchmark_group("Deserialization");

    // 1. 测试 from_bytes_borrowed (零拷贝)
    group.bench_function("from_bytes_borrowed (short, zero-copy)", |b| {
        b.iter(|| Utf8::from_bytes_borrowed(black_box(&bytes_short)))
    });

    group.bench_function("from_bytes_borrowed (long, zero-copy)", |b| {
        b.iter(|| Utf8::from_bytes_borrowed(black_box(&bytes_long)))
    });

    // 2. 测试 from_bytes_owned (有拷贝)
    group.bench_function("from_bytes_owned (short, with-copy)", |b| {
        b.iter(|| Utf8::from_bytes_owned(black_box(&bytes_short)))
    });

    group.bench_function("from_bytes_owned (long, with-copy)", |b| {
        b.iter(|| Utf8::from_bytes_owned(black_box(&bytes_long)))
    });

    group.finish();
}

/// 基准测试构造函数
fn bench_construction(c: &mut Criterion) {
    let (short_str, long_str_owned) = benchmark_data();

    let mut group = c.benchmark_group("Construction");

    group.bench_function("new_borrowed (short)", |b| {
        b.iter(|| Utf8::new_borrowed(black_box(short_str)))
    });

    // 为了公平测量，我们在循环内克隆 String
    group.bench_function("new_owned (long)", |b| {
        b.iter(|| Utf8::new_owned(black_box(long_str_owned.clone())))
    });

    group.finish();
}

// 注册所有基准测试组
criterion_group!(
    benches,
    bench_serialization,
    bench_deserialization,
    bench_construction
);
// 运行所有基准测试
criterion_main!(benches);
