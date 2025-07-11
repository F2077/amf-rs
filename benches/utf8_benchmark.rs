use amf_rs::traits::{FromBytes, ToBytes};
use amf_rs::utf8::{Length, Utf8};
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::borrow::Cow;

// 为 u16 和 u32 长度类型分别生成基准测试
macro_rules! generate_benches {
    ($len_type:ty, $suffix:ident) => {
        paste::item! {
            fn [<bench_serialization_ $suffix>](c: &mut Criterion) {
                let (short_str, long_str_owned) = benchmark_data::<$len_type>();

                let utf8_short = Utf8::<'_, $len_type>::new_borrowed(short_str).unwrap();
                let utf8_long = Utf8::<'_, $len_type>::new(Cow::Owned(long_str_owned)).unwrap();

                let mut group = c.benchmark_group(&format!("Serialization ({})", stringify!($suffix)));

                // 1. 测试 to_bytes (会分配 Vec)
                group.bench_function("to_bytes (short)", |b| b.iter(|| utf8_short.to_bytes()));
                group.bench_function("to_bytes (long)", |b| b.iter(|| utf8_long.to_bytes()));

                // 2. 测试 write_to (写入已分配的 buffer)
                let mut short_buffer = vec![0; utf8_short.bytes_size() as usize];
                group.bench_function("write_to (short)", |b| {
                    b.iter(|| utf8_short.write_bytes_to(black_box(&mut short_buffer)))
                });

                let mut long_buffer = vec![0; utf8_long.bytes_size() as usize];
                group.bench_function("write_to (long)", |b| {
                    b.iter(|| utf8_long.write_bytes_to(black_box(&mut long_buffer)))
                });

                group.finish();
            }

            fn [<bench_deserialization_ $suffix>](c: &mut Criterion) {
                let (short_str, long_str_owned) = benchmark_data::<$len_type>();

                let bytes_short = Utf8::<'_, $len_type>::new_borrowed(short_str).unwrap().to_bytes().unwrap();
                let bytes_long = Utf8::<'_, $len_type>::new(Cow::Owned(long_str_owned)).unwrap().to_bytes().unwrap();

                let mut group = c.benchmark_group(&format!("Deserialization ({})", stringify!($suffix)));

                // 1. 测试 from_bytes_borrowed (零拷贝)
                group.bench_function("from_bytes_borrowed (short, zero-copy)", |b| {
                    b.iter(|| Utf8::<'_, $len_type>::from_bytes_borrowed(black_box(&bytes_short)))
                });

                group.bench_function("from_bytes_borrowed (long, zero-copy)", |b| {
                    b.iter(|| Utf8::<'_, $len_type>::from_bytes_borrowed(black_box(&bytes_long)))
                });

                // 2. 测试 from_bytes_owned (有拷贝)
                group.bench_function("from_bytes_owned (short, with-copy)", |b| {
                    b.iter(|| Utf8::<'_, $len_type>::from_bytes_owned(black_box(&bytes_short)))
                });

                group.bench_function("from_bytes_owned (long, with-copy)", |b| {
                    b.iter(|| Utf8::<'_, $len_type>::from_bytes_owned(black_box(&bytes_long)))
                });

                group.finish();
            }

            fn [<bench_construction_ $suffix>](c: &mut Criterion) {
                let (short_str, long_str_owned) = benchmark_data::<$len_type>();

                let mut group = c.benchmark_group(&format!("Construction ({})", stringify!($suffix)));

                group.bench_function("new_borrowed (short)", |b| {
                    b.iter(|| Utf8::<'_, $len_type>::new_borrowed(black_box(short_str)))
                });

                // 为了公平测量，我们在循环内克隆 String
                group.bench_function("new_owned (long)", |b| {
                    b.iter(|| Utf8::<'_, $len_type>::new_owned(black_box(long_str_owned.clone())))
                });

                group.finish();
            }
        }
    };
}

// 为 u16 和 u32 生成基准测试函数
generate_benches!(u16, u16);
generate_benches!(u32, u32);

// 根据长度类型生成不同的测试数据
fn benchmark_data<L: Length>() -> (&'static str, String) {
    let short_str = "hello, world";

    // 对于 u16 类型，使用接近最大长度的字符串
    // 对于 u32 类型，使用更大的字符串以展示差异
    let long_length = if std::mem::size_of::<L>() == 2 {
        u16::MAX as usize - 10 // 接近 u16 最大值
    } else {
        100_000 // 10 万个字符
    };

    let long_str = "a".repeat(long_length);

    (short_str, long_str)
}

// 注册所有基准测试组
criterion_group!(
    benches,
    bench_serialization_u16,
    bench_deserialization_u16,
    bench_construction_u16,
    bench_serialization_u32,
    bench_deserialization_u32,
    bench_construction_u32,
);
// 运行所有基准测试
criterion_main!(benches);
