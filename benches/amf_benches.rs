use amf_rs::amf0::nested::Amf0TypedValue;
use amf_rs::amf0::nested::{EcmaArrayType, ObjectType};
use amf_rs::amf0::string::{LongStringType, StringType};
use amf_rs::traits::{Marshall, Unmarshall};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use indexmap::IndexMap;
use std::iter;

fn bench_string_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("String Types");

    // Prepare short string
    let short = StringType::new_from_str("hello").unwrap();
    group.bench_with_input(
        BenchmarkId::new("StringType_marshall", 5),
        &short,
        |b, s| {
            b.iter(|| s.marshall().unwrap());
        },
    );
    let short_bytes = short.marshall().unwrap();
    group.bench_with_input(
        BenchmarkId::new("StringType_unmarshall", 5),
        &short_bytes,
        |b, data| {
            b.iter(|| StringType::unmarshall(data).unwrap());
        },
    );

    // Prepare long string
    let n = u16::MAX as usize * 2;
    let long_val = iter::repeat('a').take(n).collect::<String>();
    let long = LongStringType::new_from_string(long_val).unwrap();
    group.bench_with_input(
        BenchmarkId::new("LongStringType_marshall", n),
        &long,
        |b, s| {
            b.iter(|| s.marshall().unwrap());
        },
    );
    let long_bytes = long.marshall().unwrap();
    group.bench_with_input(
        BenchmarkId::new("LongStringType_unmarshall", n),
        &long_bytes,
        |b, data| {
            b.iter(|| LongStringType::unmarshall(data).unwrap());
        },
    );

    group.finish();
}

fn bench_nested_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("Nested Types");

    // Prepare ObjectType with 100 entries
    let mut props = IndexMap::new();
    for i in 0..100 {
        let key = format!("key{}", i);
        let val = Amf0TypedValue::Number((i as f64).into());
        props.insert(key.try_into().unwrap(), val);
    }
    let object = ObjectType::new(props.clone());
    group.bench_with_input(
        BenchmarkId::new("ObjectType_marshall", 100),
        &object,
        |b, o| {
            b.iter(|| o.marshall().unwrap());
        },
    );
    let obj_bytes = object.marshall().unwrap();
    group.bench_with_input(
        BenchmarkId::new("ObjectType_unmarshall", 100),
        &obj_bytes,
        |b, data| {
            b.iter(|| ObjectType::unmarshall(data).unwrap());
        },
    );

    // Prepare EcmaArrayType with 100 entries
    let ecma = EcmaArrayType::new(props);
    group.bench_with_input(
        BenchmarkId::new("EcmaArrayType_marshall", 100),
        &ecma,
        |b, o| {
            b.iter(|| o.marshall().unwrap());
        },
    );
    let ecma_bytes = ecma.marshall().unwrap();
    group.bench_with_input(
        BenchmarkId::new("EcmaArrayType_unmarshall", 100),
        &ecma_bytes,
        |b, data| {
            b.iter(|| EcmaArrayType::unmarshall(data).unwrap());
        },
    );

    group.finish();
}

criterion_group!(benches, bench_string_types, bench_nested_types);
criterion_main!(benches);
