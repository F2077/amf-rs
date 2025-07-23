# amf-rs

🦀 **Rust implementation of Action Message Format (AMF) protocol.**

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
amf-rs = "0.1.0"
```

Then import in your crate:

```rust
use amf_rs::traits::{Marshall, Unmarshall};
use amf_rs::amf0::number::NumberType;
// ...
```

---

## Quickstart Examples

The examples below show how to marshall and unmarshall AMF0 types, along with a production‑style FLV metadata extraction.

### 1. NumberType

```rust
fn example_number_type() -> Result<(), AmfError> {
    let num = NumberType::new(3.14);
    let bytes = num.marshall()?;
    println!("[NumberType] Marshalled: {:?}", bytes);
    let (decoded, _) = NumberType::unmarshall(&bytes)?;
    println!("[NumberType] Unmarshalled value: {}\n", f64::from(decoded));
    Ok(())
}
```

### 2. BooleanType

```rust
fn example_boolean_type() -> Result<(), AmfError> {
    let flag = BooleanType::new(true);
    let bytes = flag.marshall()?;
    println!("[BooleanType] Marshalled: {:?}", bytes);
    let (decoded, _) = BooleanType::unmarshall(&bytes)?;
    println!(
        "[BooleanType] Unmarshalled value: {}\n",
        bool::from(decoded)
    );
    Ok(())
}
```

### 3. StringType & LongStringType

```rust
fn example_string_types() -> Result<(), AmfError> {
    // Short string
    let short = StringType::new_from_str("hello")?;
    let bytes = short.marshall()?;
    println!("[StringType] Marshalled: {:?}", bytes);
    let (decoded, _) = StringType::unmarshall(&bytes)?;
    println!("[StringType] Unmarshalled value: {}\n", decoded);

    // Long string
    let long_val = "a".repeat(u16::MAX as usize + 1);
    let long = LongStringType::new_from_string(long_val)?;
    let bytes = long.marshall()?;
    println!("[LongStringType] Marshalled length: {} bytes", bytes.len());
    let (_, n) = LongStringType::unmarshall(&bytes)?;
    println!("[LongStringType] Unmarshalled length: {}\n", n);
    Ok(())
}
```

### 4. NullType & UndefinedType

```rust
fn example_null_and_undefined() -> Result<(), AmfError> {
    let null = NullType::default();
    println!("[NullType] Marshalled: {:?}\n", null.marshall()?);
    let undef = UndefinedType::default();
    println!("[UndefinedType] Marshalled: {:?}\n", undef.marshall()?);
    Ok(())
}
```

### 5. Generic Amf0TypedValue

```rust
fn example_generic_typed_value() -> Result<(), AmfError> {
    let values = vec![
        Amf0TypedValue::Number(42.0.into()),
        Amf0TypedValue::Boolean(false.into()),
        Amf0TypedValue::String("test".try_into()?),
        Amf0TypedValue::LongString("world".try_into()?),
        Amf0TypedValue::Null(NullType::default()),
        Amf0TypedValue::Undefined(UndefinedType::default()),
    ];
    for v in values {
        let bytes = v.marshall()?;
        let (decoded, _) = Amf0TypedValue::unmarshall(&bytes)?;
        println!("[Amf0TypedValue] {:?} round-trip -> {:?}", v, decoded);
    }
    println!();
    Ok(())
}
```

### 6. Nested ObjectType & EcmaArrayType

```rust
fn example_nested_types() -> Result<(), AmfError> {
    let mut props = indexmap::IndexMap::new();
    props.insert("count".try_into()?, Amf0TypedValue::Number(1.23.into()));
    props.insert("active".try_into()?, Amf0TypedValue::Boolean(false.into()));

    let obj_val = Amf0TypedValue::Object(ObjectType::new(props.clone()));
    let arr_val = Amf0TypedValue::EcmaArray(EcmaArrayType::new(props));
    let examples = vec![("ObjectType", obj_val), ("EcmaArrayType", arr_val)];
    for (name, wrapped) in examples {
        let bytes = wrapped.marshall()?;
        let (decoded, _) = Amf0TypedValue::unmarshall(&bytes)?;
        println!("[{}] Unmarshalled: {}", name, decoded);
    }
    println!();
    Ok(())
}
```

### 7. FLV Metadata Extraction

```rust
fn example_extract_and_parse_flv() -> Result<(), Box<dyn std::error::Error>> {
    // Build path to examples/test.flv
    let mut flv_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    flv_path.push("examples/test.flv");

    // Extract raw ScriptData tag
    let data = extract_script_data(flv_path.to_str().unwrap())?;
    // Parse AMF0 values, skip the "onMetaData" marker
    let meta = parse_metadata(&data)?;
    println!("[FLV Metadata] {}", meta);
    Ok(())
}
```

---

## API Overview

- **Traits**: `Marshall`, `MarshallLength`, `Unmarshall`
- **Primitive Types**: `NumberType`, `BooleanType`, `StringType`, `LongStringType`, `NullType`, `UndefinedType`
- **Complex Types**: `Amf0TypedValue`, `ObjectType`, `EcmaArrayType`

---

## Examples

See [`examples/quickstart.rs`](examples/quickstart.rs).

---

## Testing

```bash
cargo test
```

---

## Reference

[Action Message Format -- AMF 0](https://rtmp.veriskope.com/pdf/amf0-file-format-specification.pdf)

---

## License

GPL-3.0

