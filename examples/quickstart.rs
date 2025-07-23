//! Quickstart Example: Extracting FLV Metadata and Marshalling AMF0 Types
//! -----------------------------------------------------------------------
//! This example demonstrates how to use the AMF0 API to marshall and unmarshall various types,
//! organized into reusable example functions, plus a production‑style FLV metadata extraction.

use std::path::PathBuf;
use std::{
    env,
    fs::File,
    io::{self, BufReader, Read, Seek, SeekFrom},
};

use amf_rs::amf0::boolean::BooleanType;
use amf_rs::amf0::marker::NullType;
use amf_rs::amf0::marker::UndefinedType;
use amf_rs::amf0::nested::{Amf0TypedValue, EcmaArrayType, ObjectType};
use amf_rs::amf0::number::NumberType;
use amf_rs::amf0::string::{LongStringType, StringType};
use amf_rs::errors::AmfError;
use amf_rs::traits::{Marshall, Unmarshall};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Single-type examples
    example_number_type()?;
    example_boolean_type()?;
    example_string_types()?;
    example_null_and_undefined()?;

    // Generic wrapper example
    example_generic_typed_value()?;

    // Nested object/array example
    example_nested_types()?;

    // Production‑style FLV metadata extraction
    example_extract_and_parse_flv()?;

    Ok(())
}

/// Example 1: Marshall and unmarshall a NumberType.
fn example_number_type() -> Result<(), AmfError> {
    let num = NumberType::new(3.14);
    let bytes = num.marshall()?;
    println!("[NumberType] Marshalled: {:?}", bytes);
    let (decoded, _) = NumberType::unmarshall(&bytes)?;
    println!("[NumberType] Unmarshalled value: {}\n", f64::from(decoded));
    Ok(())
}

/// Example 2: Marshall and unmarshall a BooleanType.
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

/// Example 3: Marshall and unmarshall StringType and LongStringType.
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

/// Example 4: Marshall and unmarshall NullType and UndefinedType.
fn example_null_and_undefined() -> Result<(), AmfError> {
    let null = NullType::default();
    println!("[NullType] Marshalled: {:?}\n", null.marshall()?);
    let undef = UndefinedType::default();
    println!("[UndefinedType] Marshalled: {:?}\n", undef.marshall()?);
    Ok(())
}

/// Example 5: Use the generic Amf0TypedValue enum to wrap multiple types.
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

/// Example 6: Demonstrate nested ObjectType and EcmaArrayType.
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

/// Example 7 (production style): Extract FLV metadata and parse AMF0 script data.
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

/// Reads an FLV file, locates the ScriptData tag, and returns its raw bytes.
fn extract_script_data<P: AsRef<str>>(path: P) -> io::Result<Vec<u8>> {
    let mut rdr = BufReader::new(File::open(path.as_ref())?);
    let mut hdr = [0u8; 9];
    rdr.read_exact(&mut hdr)?;
    if &hdr[0..3] != b"FLV" {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Not FLV"));
    }
    rdr.seek(SeekFrom::Start(13))?;
    loop {
        let mut th = [0u8; 11];
        if rdr.read_exact(&mut th).is_err() {
            break;
        }
        let len = u32::from_be_bytes([0, th[1], th[2], th[3]]);
        if th[0] == 18 {
            let mut buf = vec![0u8; len as usize];
            rdr.read_exact(&mut buf)?;
            return Ok(buf);
        }
        rdr.seek(SeekFrom::Current(len as i64 + 4))?;
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "ScriptData not found",
    ))
}

/// Parses AMF0 typed values from script data, skipping the "onMetaData" string marker.
fn parse_metadata(data: &[u8]) -> Result<String, AmfError> {
    let mut off = 0;
    let mut out = String::new();
    while off < data.len() {
        let (v, n) = Amf0TypedValue::unmarshall(&data[off..])?;
        let s = format!("{}", v);
        if s != "\"onMetaData\"" {
            out.push_str(&s);
            out.push(' ');
        }
        off += n;
    }
    Ok(out.trim().to_string())
}
