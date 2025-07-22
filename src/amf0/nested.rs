use crate::amf0::boolean::BooleanType;
use crate::amf0::marker::{NullType, UndefinedType};
use crate::amf0::number::NumberType;
use crate::amf0::object_end::ObjectEndType;
use crate::amf0::string::{LongStringType, StringType};
use crate::amf0::type_marker::TypeMarker;
use crate::amf0::unsupported::{
    DateType, MovieClipType, RecordsetType, ReferenceType, StrictArrayType, TypedObjectType,
    UnsupportedType, XmlDocumentType,
};
use crate::amf0::utf8::Utf8;
use crate::errors::AmfError;
use crate::traits::{Marshall, MarshallLength, Unmarshall};
use indexmap::IndexMap;
use std::borrow::Borrow;
use std::fmt::Display;
use std::io;
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq)]
pub enum Amf0TypedValue {
    Number(NumberType),
    Boolean(BooleanType),
    String(StringType),
    Object(ObjectType),
    MovieClip(MovieClipType),
    Null(NullType),
    Undefined(UndefinedType),
    Reference(ReferenceType),
    EcmaArray(EcmaArrayType),
    ObjectEnd(ObjectEndType),
    StrictArray(StrictArrayType),
    Date(DateType),
    LongString(LongStringType),
    Unsupported(UnsupportedType),
    Recordset(RecordsetType),
    XmlDocument(XmlDocumentType),
    TypedObject(TypedObjectType),
}

impl Marshall for Amf0TypedValue {
    fn marshall(&self) -> Result<Vec<u8>, AmfError> {
        match self {
            Amf0TypedValue::Number(v) => v.marshall(),
            Amf0TypedValue::Boolean(v) => v.marshall(),
            Amf0TypedValue::String(v) => v.marshall(),
            Amf0TypedValue::Object(v) => v.marshall(),
            Amf0TypedValue::MovieClip(v) => v.marshall(),
            Amf0TypedValue::Null(v) => v.marshall(),
            Amf0TypedValue::Undefined(v) => v.marshall(),
            Amf0TypedValue::Reference(v) => v.marshall(),
            Amf0TypedValue::EcmaArray(v) => v.marshall(),
            Amf0TypedValue::ObjectEnd(v) => v.marshall(),
            Amf0TypedValue::StrictArray(v) => v.marshall(),
            Amf0TypedValue::Date(v) => v.marshall(),
            Amf0TypedValue::LongString(v) => v.marshall(),
            Amf0TypedValue::Unsupported(v) => v.marshall(),
            Amf0TypedValue::Recordset(v) => v.marshall(),
            Amf0TypedValue::XmlDocument(v) => v.marshall(),
            Amf0TypedValue::TypedObject(v) => v.marshall(),
        }
    }
}

impl MarshallLength for Amf0TypedValue {
    fn marshall_length(&self) -> usize {
        match self {
            Amf0TypedValue::Number(v) => v.marshall_length(),
            Amf0TypedValue::Boolean(v) => v.marshall_length(),
            Amf0TypedValue::String(v) => v.marshall_length(),
            Amf0TypedValue::Object(v) => v.marshall_length(),
            Amf0TypedValue::MovieClip(v) => v.marshall_length(),
            Amf0TypedValue::Null(v) => v.marshall_length(),
            Amf0TypedValue::Undefined(v) => v.marshall_length(),
            Amf0TypedValue::Reference(v) => v.marshall_length(),
            Amf0TypedValue::EcmaArray(v) => v.marshall_length(),
            Amf0TypedValue::ObjectEnd(v) => v.marshall_length(),
            Amf0TypedValue::StrictArray(v) => v.marshall_length(),
            Amf0TypedValue::Date(v) => v.marshall_length(),
            Amf0TypedValue::LongString(v) => v.marshall_length(),
            Amf0TypedValue::Unsupported(v) => v.marshall_length(),
            Amf0TypedValue::Recordset(v) => v.marshall_length(),
            Amf0TypedValue::XmlDocument(v) => v.marshall_length(),
            Amf0TypedValue::TypedObject(v) => v.marshall_length(),
        }
    }
}

impl Unmarshall for Amf0TypedValue {
    fn unmarshall(buf: &[u8]) -> Result<(Self, usize), AmfError> {
        if buf.is_empty() {
            return Err(AmfError::Custom("Buffer is empty".to_string()));
        }
        if buf.len() >= 3 && buf[0] == 0x00 && buf[1] == 0x00 && buf[2] == 0x09 {
            return Ok((Amf0TypedValue::ObjectEnd(ObjectEndType::default()), 3));
        }

        let type_marker = TypeMarker::try_from(buf[0])?;
        match type_marker {
            TypeMarker::Number => {
                NumberType::unmarshall(buf).map(|v| (Amf0TypedValue::Number(v.0), v.1))
            }
            TypeMarker::Boolean => {
                BooleanType::unmarshall(buf).map(|v| (Amf0TypedValue::Boolean(v.0), v.1))
            }
            TypeMarker::String => {
                StringType::unmarshall(buf).map(|v| (Amf0TypedValue::String(v.0), v.1))
            }
            TypeMarker::Object => {
                ObjectType::unmarshall(buf).map(|v| (Amf0TypedValue::Object(v.0), v.1))
            }
            TypeMarker::MovieClip => {
                MovieClipType::unmarshall(buf).map(|v| (Amf0TypedValue::MovieClip(v.0), v.1))
            }
            TypeMarker::Null => NullType::unmarshall(buf).map(|v| (Amf0TypedValue::Null(v.0), v.1)),
            TypeMarker::Undefined => {
                UndefinedType::unmarshall(buf).map(|v| (Amf0TypedValue::Undefined(v.0), v.1))
            }
            TypeMarker::Reference => {
                ReferenceType::unmarshall(buf).map(|v| (Amf0TypedValue::Reference(v.0), v.1))
            }
            TypeMarker::EcmaArray => {
                EcmaArrayType::unmarshall(buf).map(|v| (Amf0TypedValue::EcmaArray(v.0), v.1))
            }
            TypeMarker::ObjectEnd => {
                panic!("cannot happen")
            }
            TypeMarker::StrictArray => {
                StrictArrayType::unmarshall(buf).map(|v| (Amf0TypedValue::StrictArray(v.0), v.1))
            }
            TypeMarker::Date => DateType::unmarshall(buf).map(|v| (Amf0TypedValue::Date(v.0), v.1)),
            TypeMarker::LongString => {
                LongStringType::unmarshall(buf).map(|v| (Amf0TypedValue::LongString(v.0), v.1))
            }
            TypeMarker::Unsupported => {
                UnsupportedType::unmarshall(buf).map(|v| (Amf0TypedValue::Unsupported(v.0), v.1))
            }
            TypeMarker::Recordset => {
                RecordsetType::unmarshall(buf).map(|v| (Amf0TypedValue::Recordset(v.0), v.1))
            }
            TypeMarker::XmlDocument => {
                XmlDocumentType::unmarshall(buf).map(|v| (Amf0TypedValue::XmlDocument(v.0), v.1))
            }
            TypeMarker::TypedObject => {
                TypedObjectType::unmarshall(buf).map(|v| (Amf0TypedValue::TypedObject(v.0), v.1))
            }
        }
    }
}

impl TryFrom<&[u8]> for Amf0TypedValue {
    type Error = AmfError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::unmarshall(value).map(|(o, _)| o)
    }
}

impl TryFrom<Vec<u8>> for Amf0TypedValue {
    type Error = AmfError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_slice())
    }
}

impl TryFrom<Amf0TypedValue> for Vec<u8> {
    type Error = AmfError;

    fn try_from(value: Amf0TypedValue) -> Result<Self, Self::Error> {
        value.marshall()
    }
}

impl Display for Amf0TypedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Amf0TypedValue::Number(v) => v.fmt(f),
            Amf0TypedValue::Boolean(v) => v.fmt(f),
            Amf0TypedValue::String(v) => v.fmt(f),
            Amf0TypedValue::Object(v) => v.fmt(f),
            Amf0TypedValue::MovieClip(v) => v.fmt(f),
            Amf0TypedValue::Null(v) => v.fmt(f),
            Amf0TypedValue::Undefined(v) => v.fmt(f),
            Amf0TypedValue::Reference(v) => v.fmt(f),
            Amf0TypedValue::EcmaArray(v) => v.fmt(f),
            Amf0TypedValue::ObjectEnd(v) => v.fmt(f),
            Amf0TypedValue::StrictArray(v) => v.fmt(f),
            Amf0TypedValue::Date(v) => v.fmt(f),
            Amf0TypedValue::LongString(v) => v.fmt(f),
            Amf0TypedValue::Unsupported(v) => v.fmt(f),
            Amf0TypedValue::Recordset(v) => v.fmt(f),
            Amf0TypedValue::XmlDocument(v) => v.fmt(f),
            Amf0TypedValue::TypedObject(v) => v.fmt(f),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NestedType<const LBW: usize, const TM: u8> {
    length: Option<u32>,
    properties: IndexMap<Utf8, Amf0TypedValue>,
    object_end: ObjectEndType,
}

impl<const LBW: usize, const TM: u8> NestedType<LBW, TM> {
    pub fn new(properties: IndexMap<Utf8, Amf0TypedValue>) -> Self {
        let length = if LBW == 4 {
            Some(properties.len() as u32)
        } else {
            None
        };
        Self {
            length,
            properties,
            object_end: ObjectEndType::default(),
        }
    }
}

impl<const LBW: usize, const TM: u8> Marshall for NestedType<LBW, TM> {
    fn marshall(&self) -> Result<Vec<u8>, AmfError> {
        let mut vec = Vec::with_capacity(self.marshall_length());
        vec.push(TM);

        if let Some(length) = self.length {
            let length_bytes = length.to_be_bytes();
            vec.extend_from_slice(&length_bytes);
        }

        self.properties
            .iter()
            .try_for_each(|(k, v)| -> io::Result<()> {
                let k_vec = k
                    .marshall()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                vec.extend_from_slice(&k_vec);
                let v_vec = v
                    .marshall()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                vec.extend_from_slice(&v_vec);
                Ok(())
            })?;

        let object_end_vec = self.object_end.marshall()?;
        vec.extend_from_slice(&object_end_vec);

        Ok(vec)
    }
}

impl<const LBW: usize, const TM: u8> MarshallLength for NestedType<LBW, TM> {
    fn marshall_length(&self) -> usize {
        let mut size = 1; // 1 byte for type marker
        size += LBW;
        let properties_bytes_size: usize = self
            .properties
            .iter()
            .map(|(k, v)| k.marshall_length() + v.marshall_length())
            .sum();
        size += properties_bytes_size;
        size += self.object_end.marshall_length();
        size
    }
}

impl<const LBW: usize, const TM: u8> Unmarshall for NestedType<LBW, TM> {
    fn unmarshall(buf: &[u8]) -> Result<(Self, usize), AmfError> {
        let required_size = 1 + LBW + 3; // 1 byte for type marker, LBW bytes(maybe 0) for optional properties length,  3 bytes for object end
        if buf.len() < required_size {
            // 1 byte for type marker, LBW bytes(maybe 0) for optional properties length,  3 bytes for object end
            return Err(AmfError::BufferTooSmall {
                want: required_size,
                got: buf.len(),
            });
        }

        if buf[0] != TM {
            return Err(AmfError::TypeMarkerValueMismatch {
                want: TM,
                got: buf[0],
            });
        }

        let mut length = 0u32;
        if LBW == 4 {
            length = u32::from_be_bytes(
                buf[1..1 + LBW]
                    .try_into()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            );
        }

        let mut properties = IndexMap::new();
        let mut offset = 1 + LBW;
        while offset < buf.len() {
            if offset <= buf.len() - 3 {
                // 找到了 object end 则退出循环
                if buf[offset] == 0x00 && buf[offset + 1] == 0x00 && buf[offset + 2] == 0x09 {
                    break;
                }
            }

            let (k, k_len) = Utf8::unmarshall(&buf[offset..])?;
            offset += k_len;
            let (v, v_len) = Amf0TypedValue::unmarshall(&buf[offset..])?;
            offset += v_len;
            properties.insert(k, v);
        }

        // 校验 object end 存在
        if buf[buf.len() - 3..] != [0x00, 0x00, 0x09] {
            return Err(AmfError::Custom(
                "Invalid object, expected object end, got end of buffer".to_string(),
            ));
        }

        // 仅在 EcmaArray 情况下(也就是 LBW == 4 的情况下)校验长度
        if LBW == 4 && properties.len() != length as usize {
            return Err(AmfError::Custom(format!(
                "Invalid properties length, want {}, got {}",
                length,
                properties.len()
            )));
        }

        let read_size = if offset == buf.len() {
            offset
        } else if offset == buf.len() - 3 {
            offset + 3
        } else {
            buf.len()
        };
        Ok((Self::new(properties), read_size))
    }
}

impl<const LBW: usize, const TM: u8> TryFrom<&[u8]> for NestedType<LBW, TM> {
    type Error = AmfError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::unmarshall(value).map(|(v, _)| v)
    }
}

impl<const LBW: usize, const TM: u8> TryFrom<Vec<u8>> for NestedType<LBW, TM> {
    type Error = AmfError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_slice())
    }
}

impl<const LBW: usize, const TM: u8> TryFrom<NestedType<LBW, TM>> for Vec<u8> {
    type Error = AmfError;

    fn try_from(value: NestedType<LBW, TM>) -> Result<Self, Self::Error> {
        value.marshall()
    }
}

impl<K, V, const LBW: usize, const TM: u8> From<IndexMap<K, V>> for NestedType<LBW, TM>
where
    K: Into<Utf8>,
    V: Into<Amf0TypedValue>,
{
    fn from(value: IndexMap<K, V>) -> Self {
        let properties = value
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        Self::new(properties)
    }
}

impl<const LBW: usize, const TM: u8> AsRef<IndexMap<Utf8, Amf0TypedValue>> for NestedType<LBW, TM> {
    fn as_ref(&self) -> &IndexMap<Utf8, Amf0TypedValue> {
        &self.properties
    }
}

impl<const LBW: usize, const TM: u8> Deref for NestedType<LBW, TM> {
    type Target = IndexMap<Utf8, Amf0TypedValue>;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<const LBW: usize, const TM: u8> Borrow<IndexMap<Utf8, Amf0TypedValue>>
    for NestedType<LBW, TM>
{
    fn borrow(&self) -> &IndexMap<Utf8, Amf0TypedValue> {
        self.as_ref()
    }
}

impl<const LBW: usize, const TM: u8> Display for NestedType<LBW, TM> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?; // 写入开头的 "{"
        // 使用 peeking iterator 来优雅地处理逗号
        let mut iter = self.properties.iter().peekable();
        while let Some((key, value)) = iter.next() {
            // 写入 "key": value
            // 注意 key 和 value 会自动使用它们自己的 Display 实现
            write!(f, "\"{}\": {}", key, value)?;
            // 如果这不是最后一个元素，就写入一个逗号和空格
            if iter.peek().is_some() {
                write!(f, ", ")?;
            }
        }
        write!(f, "}}") // 写入结尾的 "}"
    }
}

impl<const LBW: usize, const TM: u8> Default for NestedType<LBW, TM> {
    fn default() -> Self {
        Self::new(IndexMap::new())
    }
}

impl<K, V, const LBW: usize, const TM: u8> FromIterator<(K, V)> for NestedType<LBW, TM>
where
    K: Into<Utf8>,
    V: Into<Amf0TypedValue>,
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let properties = iter
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        Self::new(properties)
    }
}

impl<const LBW: usize, const TM: u8> IntoIterator for NestedType<LBW, TM> {
    type Item = (Utf8, Amf0TypedValue);
    type IntoIter = indexmap::map::IntoIter<Utf8, Amf0TypedValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.properties.into_iter()
    }
}

//	The AMF 0 Object type is used to encoded anonymous ActionScript objects. Any typed
//	object that does not have a registered class should be treated as an anonymous
//	ActionScript object. If the same object instance appears in an object graph it should be
//	sent by reference using an AMF 0.
//	Use the reference type to reduce redundant information from being serialized and infinite
//	loops from cyclical references.
pub type ObjectType = NestedType<0, { TypeMarker::Object as u8 }>;

// An ECMA Array or 'associative' Array is used when an ActionScript Array contains non-ordinal indices.
// This type is considered a complex type and thus reoccurring instancescan be sent by reference.
// All indices. ordinal or otherwise, are treated as string keysinstead of integers.
// For the purposes of serialization this type is very similar to ananonymous Obiect.
pub type EcmaArrayType = NestedType<4, { TypeMarker::EcmaArray as u8 }>;

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    // Helper function to create a sample IndexMap for NestedType tests
    fn sample_properties() -> IndexMap<Utf8, Amf0TypedValue> {
        let mut props = IndexMap::new();
        props.insert(
            Utf8::new_from_str("key1").unwrap(),
            Amf0TypedValue::Number(NumberType::new(42.0)),
        );
        props.insert(
            Utf8::new_from_str("key2").unwrap(),
            Amf0TypedValue::String(StringType::try_from("value").unwrap()),
        );
        props
    }

    // Tests for Amf0TypedValue variants
    #[test]
    fn test_number() {
        let original = Amf0TypedValue::Number(NumberType::new(42.0));
        let marshalled = original.marshall().unwrap();
        let (unmarshalled, _) = Amf0TypedValue::unmarshall(&marshalled).unwrap();
        assert_eq!(original, unmarshalled);
    }

    #[test]
    fn test_boolean() {
        let original = Amf0TypedValue::Boolean(BooleanType::new(true));
        let marshalled = original.marshall().unwrap();
        let (unmarshalled, _) = Amf0TypedValue::unmarshall(&marshalled).unwrap();
        assert_eq!(original, unmarshalled);
    }

    #[test]
    fn test_string() {
        let original = Amf0TypedValue::String(StringType::new_from_str("hello").unwrap());
        let marshalled = original.marshall().unwrap();
        let (unmarshalled, _) = Amf0TypedValue::unmarshall(&marshalled).unwrap();
        assert_eq!(original, unmarshalled);
    }

    #[test]
    fn test_object() {
        let props = sample_properties();
        let object_type = ObjectType::new(props);
        let original = Amf0TypedValue::Object(object_type);
        let marshalled = original.marshall().unwrap();
        let (unmarshalled, _) = Amf0TypedValue::unmarshall(&marshalled).unwrap();
        assert_eq!(original, unmarshalled);
    }

    #[test]
    fn test_null() {
        let original = Amf0TypedValue::Null(NullType);
        let marshalled = original.marshall().unwrap();
        let (unmarshalled, _) = Amf0TypedValue::unmarshall(&marshalled).unwrap();
        assert_eq!(original, unmarshalled);
    }

    #[test]
    fn test_undefined() {
        let original = Amf0TypedValue::Undefined(UndefinedType);
        let marshalled = original.marshall().unwrap();
        let (unmarshalled, _) = Amf0TypedValue::unmarshall(&marshalled).unwrap();
        assert_eq!(original, unmarshalled);
    }

    #[test]
    fn test_ecma_array() {
        let props = sample_properties();
        let ecma_array_type = EcmaArrayType::new(props);
        let original = Amf0TypedValue::EcmaArray(ecma_array_type);
        let marshalled = original.marshall().unwrap();
        let (unmarshalled, _) = Amf0TypedValue::unmarshall(&marshalled).unwrap();
        assert_eq!(original, unmarshalled);
    }

    #[test]
    fn test_object_end() {
        let original = Amf0TypedValue::ObjectEnd(ObjectEndType::default());
        let marshalled = original.marshall().unwrap();
        let (unmarshalled, _) = Amf0TypedValue::unmarshall(&marshalled).unwrap();
        assert_eq!(original, unmarshalled);
    }

    #[test]
    fn test_long_string() {
        let original =
            Amf0TypedValue::LongString(LongStringType::new_from_string("a".repeat(65536)).unwrap());
        let marshalled = original.marshall().unwrap();
        let (unmarshalled, _) = Amf0TypedValue::unmarshall(&marshalled).unwrap();
        assert_eq!(original, unmarshalled);
    }

    // Tests for Clone and PartialEq on Amf0TypedValue
    #[test]
    fn test_amf0_typed_value_clone() {
        let original = Amf0TypedValue::Object(ObjectType::new(sample_properties()));
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_amf0_typed_value_partial_eq() {
        let num1 = Amf0TypedValue::Number(NumberType::new(42.0));
        let num2 = Amf0TypedValue::Number(NumberType::new(42.0));
        let num3 = Amf0TypedValue::Number(NumberType::new(43.0));
        assert_eq!(num1, num2);
        assert_ne!(num1, num3);

        let obj = Amf0TypedValue::Object(ObjectType::new(sample_properties()));
        let bool_val = Amf0TypedValue::Boolean(BooleanType::new(false));
        assert_ne!(obj, bool_val);
    }

    // Tests for NestedType (ObjectType and EcmaArrayType)
    #[test]
    fn test_object_type() {
        let props = sample_properties();
        let original = ObjectType::new(props);
        let marshalled = original.marshall().unwrap();
        let (unmarshalled, _) = ObjectType::unmarshall(&marshalled).unwrap();
        assert_eq!(original, unmarshalled);
    }

    #[test]
    fn test_ecma_array_type() {
        let props = sample_properties();
        let original = EcmaArrayType::new(props);
        let marshalled = original.marshall().unwrap();
        let (unmarshalled, _) = EcmaArrayType::unmarshall(&marshalled).unwrap();
        assert_eq!(original, unmarshalled);
    }

    #[test]
    fn test_nested_type_clone() {
        let original = ObjectType::new(sample_properties());
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_nested_type_partial_eq() {
        let props1 = sample_properties();
        let obj1 = ObjectType::new(props1.clone());
        let obj2 = ObjectType::new(props1);
        assert_eq!(obj1, obj2);

        let mut props2 = IndexMap::new();
        props2.insert(
            Utf8::try_from("key1").unwrap(),
            Amf0TypedValue::Number(NumberType::new(43.0)),
        );
        let obj3 = ObjectType::new(props2);
        assert_ne!(obj1, obj3);
    }

    // Error case tests
    #[test]
    fn test_unmarshall_invalid_type_marker() {
        let buf = [0xff]; // Invalid type marker
        let result = Amf0TypedValue::unmarshall(&buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_type_buffer_too_small() {
        let buf = [TypeMarker::Object as u8];
        let result = ObjectType::unmarshall(&buf);
        assert!(matches!(result, Err(AmfError::BufferTooSmall { .. })));
    }
}
