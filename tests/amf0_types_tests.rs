use amf_rs::amf0::{
    BooleanType, ECMAArrayType, LongStringType, NullType, NumberType, ObjectEndType, ObjectType,
    StringType, UndefinedType,
};
use amf_rs::traits::{FromBytes, ToBytes};
use amf_rs::type_marker::TypeMarker;
use amf_rs::utf8::{AmfUtf8, Utf8};
use indexmap::IndexMap;

#[test]
fn test_number_type() {
    // Test basic functionality
    let num = NumberType::new(3.14159);
    assert_eq!(*num, 3.14159);
    assert_eq!(format!("{}", num), "3.14159");

    // Test serialization
    let bytes = num.to_bytes().unwrap();
    assert_eq!(bytes.len(), 9);
    assert_eq!(bytes[0], TypeMarker::Number as u8);
    assert_eq!(f64::from_be_bytes(bytes[1..9].try_into().unwrap()), 3.14159);

    // Test deserialization
    let (parsed, consumed) = NumberType::from_bytes(&bytes).unwrap();
    assert_eq!(consumed, 9);
    assert_eq!(*parsed, 3.14159);

    // Test error cases
    assert!(NumberType::from_bytes(&[0x01]).is_err()); // Too short
    assert!(NumberType::from_bytes(&[0x02, 0, 0, 0, 0, 0, 0, 0, 0]).is_err()); // Wrong marker
}

#[test]
fn test_boolean_type() {
    // Test true value
    let true_val = BooleanType::new(true);
    assert_eq!(
        true_val.to_bytes().unwrap(),
        vec![TypeMarker::Boolean as u8, 1]
    );

    // Test false value
    let false_val = BooleanType::new(false);
    assert_eq!(
        false_val.to_bytes().unwrap(),
        vec![TypeMarker::Boolean as u8, 0]
    );

    // Test serialization/deserialization
    for &value in &[true, false] {
        let bool_type = BooleanType::new(value);
        let bytes = bool_type.to_bytes().unwrap();
        assert_eq!(bytes.len(), 2);

        let (parsed, consumed) = BooleanType::from_bytes(&bytes).unwrap();
        assert_eq!(consumed, 2);
        assert_eq!(*parsed, value);
    }

    // Test error cases
    assert!(BooleanType::from_bytes(&[0x01]).is_err()); // Too short
    assert!(BooleanType::from_bytes(&[0x03, 1]).is_err()); // Wrong marker
}

#[test]
fn test_string_type() {
    // Test basic functionality
    let s = Utf8::from(AmfUtf8::try_from("hello").unwrap());
    let str_type = StringType::new(s);
    assert_eq!(str_type.bytes_size(), 1 + 2 + 5); // Marker + length + content

    // Test serialization
    let bytes = str_type.to_bytes().unwrap();
    assert_eq!(bytes[0], TypeMarker::String as u8);
    assert_eq!(&bytes[3..], b"hello");

    // Test deserialization
    let (parsed, consumed) = StringType::from_bytes(&bytes).unwrap();
    assert_eq!(consumed, 8);
    assert_eq!(parsed.as_ref().as_ref(), "hello");

    // Test empty string
    let empty = StringType::new(AmfUtf8::Utf8(Utf8::from("")));
    let bytes = empty.to_bytes().unwrap();
    assert_eq!(bytes, vec![0x02, 0x00, 0x00]);
}

#[test]
fn test_long_string_type() {
    // Create a long string (> 65KB would be ideal but impractical in test)
    let data = "a".repeat(1000);
    let long_str = LongStringType::new(AmfUtf8::LongUtf8(Utf8::from(&data)));

    // Test serialization
    let bytes = long_str.to_bytes().unwrap();
    assert_eq!(bytes[0], TypeMarker::LongString as u8);
    let len_bytes: [u8; 4] = bytes[1..5].try_into().unwrap();
    assert_eq!(u32::from_be_bytes(len_bytes), 1000);

    // Test deserialization
    let (parsed, consumed) = LongStringType::from_bytes(&bytes).unwrap();
    assert_eq!(consumed, 1 + 4 + 1000);
    assert_eq!(parsed.as_ref().as_ref(), data);
}

#[test]
fn test_object_end_type() {
    let obj_end = ObjectEndType::new();

    // Test serialization
    let bytes = obj_end.to_bytes().unwrap();
    assert_eq!(bytes, vec![0x00, 0x00, 0x09]);

    // Test deserialization
    let (parsed, consumed) = ObjectEndType::from_bytes(&bytes).unwrap();
    assert_eq!(consumed, 3);
    assert_eq!(parsed.as_ref().as_ref().as_ref(), "");
}

#[test]
fn test_object_type() {
    // Create object with number properties
    let mut properties = indexmap! {
        Utf8::from("x") => NumberType::new(10.0).into_amf_type(),
        Utf8::from("y") => NumberType::new(20.0).into_amf_type(),
    };

    let obj = ObjectType::new(properties.clone());

    // Test serialization
    let bytes = obj.to_bytes().unwrap();
    assert_eq!(bytes[0], TypeMarker::Object as u8);

    // Test deserialization
    let (parsed, consumed) = ObjectType::from_bytes(&bytes).unwrap();
    assert_eq!(parsed.len(), 2);
    assert_eq!(
        *parsed.get(&Utf8::from("x")).unwrap(),
        NumberType::new(10.0)
    );
    assert_eq!(
        *parsed.get(&Utf8::from("y")).unwrap(),
        NumberType::new(20.0)
    );
}

#[test]
fn test_ecma_array_type() {
    // Create ECMA array
    let mut properties = indexmap! {
        Utf8::from("name") => StringType::new(AmfUtf8::Utf8(Utf8::from("John"))).into_amf_type(),
        Utf8::from("age") => NumberType::new(30.0).into_amf_type(),
    };

    let ecma = ECMAArrayType::new(properties.clone());

    // Test serialization
    let bytes = ecma.to_bytes().unwrap();
    assert_eq!(bytes[0], TypeMarker::EcmaArray as u8);
    assert_eq!(&bytes[1..5], &(2u32).to_be_bytes()); // Length prefix

    // Test deserialization
    let (parsed, consumed) = ECMAArrayType::from_bytes(&bytes).unwrap();
    assert_eq!(parsed.len(), 2);
    assert_eq!(
        *parsed.get(&Utf8::from("name")).unwrap(),
        StringType::new(AmfUtf8::Utf8(Utf8::from("John")))
    );
}

#[test]
fn test_null_type() {
    let null = NullType;

    // Test serialization
    let bytes = null.to_bytes().unwrap();
    assert_eq!(bytes, vec![TypeMarker::Null as u8]);

    // Test deserialization
    let (parsed, consumed) = NullType::from_bytes(&bytes).unwrap();
    assert_eq!(consumed, 1);
    assert_eq!(parsed, NullType);
}

#[test]
fn test_undefined_type() {
    let undef = UndefinedType;

    // Test serialization
    let bytes = undef.to_bytes().unwrap();
    assert_eq!(bytes, vec![TypeMarker::Undefined as u8]);

    // Test deserialization
    let (parsed, consumed) = UndefinedType::from_bytes(&bytes).unwrap();
    assert_eq!(consumed, 1);
    assert_eq!(parsed, UndefinedType);
}

// Helper trait for converting to AmfType object
trait IntoAmfType {
    fn into_amf_type(self) -> Box<dyn AmfType>;
}

impl IntoAmfType for NumberType {
    fn into_amf_type(self) -> Box<dyn AmfType> {
        Box::new(self)
    }
}

impl IntoAmfType for StringType<'_> {
    fn into_amf_type(self) -> Box<dyn AmfType> {
        Box::new(self)
    }
}
