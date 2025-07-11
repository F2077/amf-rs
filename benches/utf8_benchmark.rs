use amf_rs::traits::{FromBytes, FromBytesRef, ToBytes};
use amf_rs::utf8::{Length, Utf8};
use criterion::{Criterion, black_box, criterion_group, criterion_main};

/// 生成测试数据：短串和长串
fn benchmark_data<L: Length>() -> (&'static str, String) {
    let short_str = "hello, world";
    let long_length = if std::mem::size_of::<L>() == 2 {
        u16::MAX as usize - 10
    } else {
        100_000
    };
    let long_str = "a".repeat(long_length);
    (short_str, long_str)
}

/// ======================
/// u16 长度类型的基准
/// ======================

pub fn bench_serialization_u16(c: &mut Criterion) {
    let (short_str, long_owned) = benchmark_data::<u16>();
    let utf8_short = Utf8::<'_, u16>::new_borrowed(short_str).unwrap();
    let utf8_long = Utf8::<'_, u16>::new_owned(long_owned).unwrap();

    let mut group = c.benchmark_group("Serialization (u16)");
    // to_bytes
    group.bench_function("to_bytes (short)", |b| b.iter(|| utf8_short.to_bytes()));
    group.bench_function("to_bytes (long)", |b| b.iter(|| utf8_long.to_bytes()));
    // write_bytes_to
    let mut buf_short = vec![0; utf8_short.bytes_size() as usize];
    group.bench_function("write_bytes_to (short)", |b| {
        b.iter(|| utf8_short.write_bytes_to(black_box(&mut buf_short)))
    });
    let mut buf_long = vec![0; utf8_long.bytes_size() as usize];
    group.bench_function("write_bytes_to (long)", |b| {
        b.iter(|| utf8_long.write_bytes_to(black_box(&mut buf_long)))
    });
    group.finish();
}

pub fn bench_deserialization_u16(c: &mut Criterion) {
    let (short_str, long_owned) = benchmark_data::<u16>();
    let bytes_short = Utf8::<'_, u16>::new_borrowed(short_str)
        .unwrap()
        .to_bytes()
        .unwrap();
    let bytes_long = Utf8::<'_, u16>::new_owned(long_owned)
        .unwrap()
        .to_bytes()
        .unwrap();

    let mut group = c.benchmark_group("Deserialization (u16)");
    // zero-copy borrow
    group.bench_function("from_bytes_ref (short)", |b| {
        b.iter(|| Utf8::<'_, u16>::from_bytes_ref(black_box(&bytes_short)))
    });
    group.bench_function("from_bytes_ref (long)", |b| {
        b.iter(|| Utf8::<'_, u16>::from_bytes_ref(black_box(&bytes_long)))
    });
    // owned-copy
    group.bench_function("from_bytes (short)", |b| {
        b.iter(|| Utf8::<'_, u16>::from_bytes(black_box(&bytes_short)))
    });
    group.bench_function("from_bytes (long)", |b| {
        b.iter(|| Utf8::<'_, u16>::from_bytes(black_box(&bytes_long)))
    });
    group.finish();
}

pub fn bench_construction_u16(c: &mut Criterion) {
    let (short_str, long_owned) = benchmark_data::<u16>();
    let mut group = c.benchmark_group("Construction (u16)");
    group.bench_function("new_borrowed (short)", |b| {
        b.iter(|| Utf8::<'_, u16>::new_borrowed(black_box(short_str)))
    });
    group.bench_function("new_owned (long)", |b| {
        b.iter(|| Utf8::<'_, u16>::new_owned(black_box(long_owned.clone())))
    });
    group.finish();
}

/// ======================
/// u32 长度类型的基准
/// ======================

pub fn bench_serialization_u32(c: &mut Criterion) {
    let (short_str, long_owned) = benchmark_data::<u32>();
    let utf8_short = Utf8::<'_, u32>::new_borrowed(short_str).unwrap();
    let utf8_long = Utf8::<'_, u32>::new_owned(long_owned).unwrap();

    let mut group = c.benchmark_group("Serialization (u32)");
    group.bench_function("to_bytes (short)", |b| b.iter(|| utf8_short.to_bytes()));
    group.bench_function("to_bytes (long)", |b| b.iter(|| utf8_long.to_bytes()));
    let mut buf_short = vec![0; utf8_short.bytes_size() as usize];
    group.bench_function("write_bytes_to (short)", |b| {
        b.iter(|| utf8_short.write_bytes_to(black_box(&mut buf_short)))
    });
    let mut buf_long = vec![0; utf8_long.bytes_size() as usize];
    group.bench_function("write_bytes_to (long)", |b| {
        b.iter(|| utf8_long.write_bytes_to(black_box(&mut buf_long)))
    });
    group.finish();
}

pub fn bench_deserialization_u32(c: &mut Criterion) {
    let (short_str, long_owned) = benchmark_data::<u32>();
    let bytes_short = Utf8::<'_, u32>::new_borrowed(short_str)
        .unwrap()
        .to_bytes()
        .unwrap();
    let bytes_long = Utf8::<'_, u32>::new_owned(long_owned)
        .unwrap()
        .to_bytes()
        .unwrap();

    let mut group = c.benchmark_group("Deserialization (u32)");
    group.bench_function("from_bytes_ref (short)", |b| {
        b.iter(|| Utf8::<'_, u32>::from_bytes_ref(black_box(&bytes_short)))
    });
    group.bench_function("from_bytes_ref (long)", |b| {
        b.iter(|| Utf8::<'_, u32>::from_bytes_ref(black_box(&bytes_long)))
    });
    group.bench_function("from_bytes (short)", |b| {
        b.iter(|| Utf8::<'_, u32>::from_bytes(black_box(&bytes_short)))
    });
    group.bench_function("from_bytes (long)", |b| {
        b.iter(|| Utf8::<'_, u32>::from_bytes(black_box(&bytes_long)))
    });
    group.finish();
}

pub fn bench_construction_u32(c: &mut Criterion) {
    let (short_str, long_owned) = benchmark_data::<u32>();
    let mut group = c.benchmark_group("Construction (u32)");
    group.bench_function("new_borrowed (short)", |b| {
        b.iter(|| Utf8::<'_, u32>::new_borrowed(black_box(short_str)))
    });
    group.bench_function("new_owned (long)", |b| {
        b.iter(|| Utf8::<'_, u32>::new_owned(black_box(long_owned.clone())))
    });
    group.finish();
}

// 注册并运行
criterion_group!(
    benches,
    bench_serialization_u16,
    bench_deserialization_u16,
    bench_construction_u16,
    bench_serialization_u32,
    bench_deserialization_u32,
    bench_construction_u32,
);
criterion_main!(benches);
