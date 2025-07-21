use crate::amf0::object_end::ObjectEndType;
use crate::amf0::type_marker::TypeMarker;
use crate::amf0::utf8::Utf8;
use crate::errors::AmfError;
use crate::traits::{Marshall, MarshallLength, Unmarshall};
use indexmap::IndexMap;
use std::borrow::Borrow;
use std::fmt::Display;
use std::io;
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NestedType<
    T: Marshall + MarshallLength + Unmarshall,
    const LENGTH_BYTE_WIDTH: usize,
    const TYPE_MARKER: u8,
> {
    length: Option<u32>,
    properties: IndexMap<Utf8, T>,
    object_end: ObjectEndType,
}

impl<
    T: Marshall + MarshallLength + Unmarshall,
    const LENGTH_BYTE_WIDTH: usize,
    const TYPE_MARKER: u8,
> NestedType<T, LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    pub fn new(properties: IndexMap<Utf8, T>) -> Self {
        let length = if LENGTH_BYTE_WIDTH == 4 {
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

impl<
    T: Marshall + MarshallLength + Unmarshall,
    const LENGTH_BYTE_WIDTH: usize,
    const TYPE_MARKER: u8,
> Marshall for NestedType<T, LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    fn marshall(&self) -> Result<Vec<u8>, AmfError> {
        let mut vec = Vec::with_capacity(self.marshall_length());
        vec.push(TYPE_MARKER);

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

impl<
    T: Marshall + MarshallLength + Unmarshall,
    const LENGTH_BYTE_WIDTH: usize,
    const TYPE_MARKER: u8,
> MarshallLength for NestedType<T, LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    fn marshall_length(&self) -> usize {
        let mut size = 1; // 1 byte for type marker
        size += LENGTH_BYTE_WIDTH;
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

impl<
    T: Marshall + MarshallLength + Unmarshall,
    const LENGTH_BYTE_WIDTH: usize,
    const TYPE_MARKER: u8,
> Unmarshall for NestedType<T, LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    fn unmarshall(buf: &[u8]) -> Result<(Self, usize), AmfError> {
        let required_size = 1 + LENGTH_BYTE_WIDTH + 3;
        if buf.len() < required_size {
            // 1 byte for type marker, LENGTH_BYTE_WIDTH bytes(maybe 0) for optional properties length,  3 bytes for object end
            return Err(AmfError::BufferTooSmall {
                want: required_size,
                got: buf.len(),
            });
        }

        if buf[0] != TYPE_MARKER {
            return Err(AmfError::TypeMarkerValueMismatch {
                want: TYPE_MARKER,
                got: buf[0],
            });
        }

        let mut length = 0u32;
        if LENGTH_BYTE_WIDTH == 4 {
            length = u32::from_be_bytes(
                buf[1..1 + LENGTH_BYTE_WIDTH]
                    .try_into()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            );
        }

        let mut properties = IndexMap::new();
        let mut offset = 1 + LENGTH_BYTE_WIDTH;
        while offset < buf.len() {
            if offset <= buf.len() - 3 {
                // 找到了 object end 则退出循环
                if buf[offset] == 0x00 && buf[offset + 1] == 0x00 && buf[offset + 2] == 0x09 {
                    break;
                }
            }

            let (k, k_len) = Utf8::unmarshall(&buf[offset..])?;
            offset += k_len;
            let (v, v_len) = T::unmarshall(&buf[offset..])?;
            offset += v_len;
            properties.insert(k, v);
        }

        // 校验 object end 存在
        if buf[buf.len() - 3..] != [0x00, 0x00, 0x09] {
            return Err(AmfError::Custom(
                "Invalid object, expected object end, got end of buffer".to_string(),
            ));
        }

        // 仅在 EcmaArray 情况下校验长度
        if LENGTH_BYTE_WIDTH == 4 && properties.len() != length as usize {
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

impl<
    T: Marshall + MarshallLength + Unmarshall,
    const LENGTH_BYTE_WIDTH: usize,
    const TYPE_MARKER: u8,
> TryFrom<&[u8]> for NestedType<T, LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    type Error = AmfError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::unmarshall(value).map(|(v, _)| v)
    }
}

impl<
    K,
    V,
    T: Marshall + MarshallLength + Unmarshall,
    const LENGTH_BYTE_WIDTH: usize,
    const TYPE_MARKER: u8,
> From<IndexMap<K, V>> for NestedType<T, LENGTH_BYTE_WIDTH, TYPE_MARKER>
where
    K: Into<Utf8>,
    V: Into<T>,
{
    fn from(value: IndexMap<K, V>) -> Self {
        let properties = value
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        Self::new(properties)
    }
}

impl<
    T: Marshall + MarshallLength + Unmarshall,
    const LENGTH_BYTE_WIDTH: usize,
    const TYPE_MARKER: u8,
> AsRef<IndexMap<Utf8, T>> for NestedType<T, LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    fn as_ref(&self) -> &IndexMap<Utf8, T> {
        &self.properties
    }
}

impl<
    T: Marshall + MarshallLength + Unmarshall,
    const LENGTH_BYTE_WIDTH: usize,
    const TYPE_MARKER: u8,
> Deref for NestedType<T, LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    type Target = IndexMap<Utf8, T>;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<
    T: Marshall + MarshallLength + Unmarshall,
    const LENGTH_BYTE_WIDTH: usize,
    const TYPE_MARKER: u8,
> Borrow<IndexMap<Utf8, T>> for NestedType<T, LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    fn borrow(&self) -> &IndexMap<Utf8, T> {
        self.as_ref()
    }
}

impl<
    T: Marshall + MarshallLength + Unmarshall + Display,
    const LENGTH_BYTE_WIDTH: usize,
    const TYPE_MARKER: u8,
> Display for NestedType<T, LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
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

impl<
    T: Marshall + MarshallLength + Unmarshall + Display,
    const LENGTH_BYTE_WIDTH: usize,
    const TYPE_MARKER: u8,
> Default for NestedType<T, LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    fn default() -> Self {
        Self::new(IndexMap::new())
    }
}

impl<
    K,
    V,
    T: Marshall + MarshallLength + Unmarshall + Display,
    const LENGTH_BYTE_WIDTH: usize,
    const TYPE_MARKER: u8,
> FromIterator<(K, V)> for NestedType<T, LENGTH_BYTE_WIDTH, TYPE_MARKER>
where
    K: Into<Utf8>,
    V: Into<T>,
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let properties = iter
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        Self::new(properties)
    }
}

impl<
    T: Marshall + MarshallLength + Unmarshall + Display,
    const LENGTH_BYTE_WIDTH: usize,
    const TYPE_MARKER: u8,
> IntoIterator for NestedType<T, LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    type Item = (Utf8, T);
    type IntoIter = indexmap::map::IntoIter<Utf8, T>;

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
pub type ObjectType<T: Marshall + MarshallLength + Unmarshall> =
    NestedType<T, 0, { TypeMarker::Object as u8 }>;

// An ECMA Array or 'associative' Array is used when an ActionScript Array contains non-ordinal indices.
// This type is considered a complex type and thus reoccurring instancescan be sent by reference.
// All indices. ordinal or otherwise, are treated as string keysinstead of integers.
// For the purposes of serialization this type is very similar to ananonymous Obiect.
pub type ECMAArrayType<T: Marshall + MarshallLength + Unmarshall> =
    NestedType<T, 4, { TypeMarker::EcmaArray as u8 }>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amf0::number::NumberType;
    use crate::amf0::type_marker::TypeMarker;
    use indexmap::IndexMap;

    // 测试辅助函数：创建测试对象
    fn create_test_object() -> ObjectType<NumberType> {
        let mut properties = IndexMap::new();
        properties.insert(Utf8::new("a".into()).unwrap(), NumberType::new(1.0));
        properties.insert(Utf8::new("b".into()).unwrap(), NumberType::new(2.0));
        ObjectType::new(properties)
    }

    // 测试辅助函数：创建测试 ECMA 数组
    fn create_test_ecma_array() -> ECMAArrayType<NumberType> {
        let mut properties = IndexMap::new();
        properties.insert(Utf8::new("a".into()).unwrap(), NumberType::new(1.0));
        properties.insert(Utf8::new("b".into()).unwrap(), NumberType::new(2.0));
        ECMAArrayType::new(properties)
    }

    // ObjectType 测试组
    #[test]
    fn test_object_new() {
        let obj = create_test_object();
        assert_eq!(obj.len(), 2);
        assert_eq!(obj["a"], NumberType::new(1.0));
        assert_eq!(obj["b"], NumberType::new(2.0));
    }

    #[test]
    fn test_object_marshall() {
        let obj = create_test_object();
        let data = obj.marshall().unwrap();

        // 验证类型标记
        assert_eq!(data[0], TypeMarker::Object as u8);

        // 验证序列化结构
        let expected = vec![
            TypeMarker::Object as u8, // 对象标记
            0x00,
            0x01,
            b'a', // 键 "a"
            TypeMarker::Number as u8,
            0x3F,
            0xF0,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00, // 值 1.0
            0x00,
            0x01,
            b'b', // 键 "b"
            TypeMarker::Number as u8,
            0x40,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00, // 值 2.0
            0x00,
            0x00,
            0x09, // 对象结束标记
        ];

        assert_eq!(data, expected);
    }

    #[test]
    fn test_object_marshall_length() {
        let obj = create_test_object();
        // 1 (marker) + 2*3 (keys: 2 bytes length + 1 char) + 2*9 (values) + 3 (object end)
        assert_eq!(obj.marshall_length(), 1 + (3 + 9) * 2 + 3);
    }

    #[test]
    fn test_object_unmarshall() {
        let data = vec![
            TypeMarker::Object as u8, // 对象标记
            // 键 "a"
            0x00,
            0x01,
            b'a',
            // 值 1.0
            TypeMarker::Number as u8,
            0x3F,
            0xF0,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            // 键 "b"
            0x00,
            0x01,
            b'b',
            // 值 2.0
            TypeMarker::Number as u8,
            0x40,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            // 对象结束
            0x00,
            0x00,
            0x09,
        ];

        let (obj, bytes_read) = ObjectType::<NumberType>::unmarshall(&data).unwrap();
        assert_eq!(bytes_read, data.len());
        assert_eq!(obj.len(), 2);
        assert_eq!(obj["a"], NumberType::new(1.0));
        assert_eq!(obj["b"], NumberType::new(2.0));
    }

    #[test]
    fn test_object_unmarshall_empty() {
        let data = vec![
            TypeMarker::Object as u8, // 对象标记
            // 对象结束
            0x00,
            0x00,
            0x09,
        ];

        let (obj, bytes_read) = ObjectType::<NumberType>::unmarshall(&data).unwrap();
        assert_eq!(bytes_read, data.len());
        assert_eq!(obj.len(), 0);
    }

    #[test]
    fn test_object_unmarshall_invalid_marker() {
        let data = vec![
            TypeMarker::Null as u8, // 错误的标记
            0x00,
            0x01,
            b'a',
            TypeMarker::Number as u8,
            0x3F,
            0xF0,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x09,
        ];

        let result = ObjectType::<NumberType>::unmarshall(&data);
        assert!(matches!(
            result,
            Err(AmfError::TypeMarkerValueMismatch {
                want: 0x03,
                got: 0x05
            })
        ));
    }

    #[test]
    fn test_object_unmarshall_missing_end() {
        let data = vec![
            TypeMarker::Object as u8,
            0x00,
            0x01,
            b'a',
            TypeMarker::Number as u8,
            0x3F,
            0xF0,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            // 缺少对象结束标记
        ];

        let result = ObjectType::<NumberType>::unmarshall(&data);
        assert!(matches!(result, Err(AmfError::Custom(_))));
    }

    // ECMAArrayType 测试组
    #[test]
    fn test_ecma_array_new() {
        let arr = create_test_ecma_array();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr["a"], NumberType::new(1.0));
        assert_eq!(arr["b"], NumberType::new(2.0));
    }

    #[test]
    fn test_ecma_array_marshall() {
        let arr = create_test_ecma_array();
        let data = arr.marshall().unwrap();

        // 验证类型标记
        assert_eq!(data[0], TypeMarker::EcmaArray as u8);

        // 验证长度字段 (4 字节)
        assert_eq!(&data[1..5], [0x00, 0x00, 0x00, 0x02]);

        // 验证序列化结构
        let expected = vec![
            TypeMarker::EcmaArray as u8, // ECMA 数组标记
            0x00,
            0x00,
            0x00,
            0x02, // 长度 2
            // 键 "a"
            0x00,
            0x01,
            b'a',
            // 值 1.0
            TypeMarker::Number as u8,
            0x3F,
            0xF0,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            // 键 "b"
            0x00,
            0x01,
            b'b',
            // 值 2.0
            TypeMarker::Number as u8,
            0x40,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            // 对象结束
            0x00,
            0x00,
            0x09,
        ];

        assert_eq!(data, expected);
    }

    #[test]
    fn test_ecma_array_marshall_length() {
        let arr = create_test_ecma_array();
        // 1 (marker) + 4 (length) + 2*3 (keys) + 2*9 (values) + 3 (object end)
        assert_eq!(arr.marshall_length(), 1 + 4 + (3 + 9) * 2 + 3);
    }

    #[test]
    fn test_ecma_array_unmarshall() {
        let data = vec![
            TypeMarker::EcmaArray as u8, // ECMA 数组标记
            0x00,
            0x00,
            0x00,
            0x02, // 长度 2
            // 键 "a"
            0x00,
            0x01,
            b'a',
            // 值 1.0
            TypeMarker::Number as u8,
            0x3F,
            0xF0,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            // 键 "b"
            0x00,
            0x01,
            b'b',
            // 值 2.0
            TypeMarker::Number as u8,
            0x40,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            // 对象结束
            0x00,
            0x00,
            0x09,
        ];

        let (arr, bytes_read) = ECMAArrayType::<NumberType>::unmarshall(&data).unwrap();
        assert_eq!(bytes_read, data.len());
        assert_eq!(arr.len(), 2);
        assert_eq!(arr["a"], NumberType::new(1.0));
        assert_eq!(arr["b"], NumberType::new(2.0));
    }

    #[test]
    fn test_ecma_array_unmarshall_length_mismatch() {
        let data = vec![
            TypeMarker::EcmaArray as u8, // ECMA 数组标记
            0x00,
            0x00,
            0x00,
            0x03, // 长度 3 (但实际只有2个属性)
            // 键 "a"
            0x00,
            0x01,
            b'a',
            // 值 1.0
            TypeMarker::Number as u8,
            0x3F,
            0xF0,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            // 键 "b"
            0x00,
            0x01,
            b'b',
            // 值 2.0
            TypeMarker::Number as u8,
            0x40,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            // 对象结束
            0x00,
            0x00,
            0x09,
        ];

        let result = ECMAArrayType::<NumberType>::unmarshall(&data);
        assert!(matches!(result, Err(AmfError::Custom(_))));
    }

    // 通用功能测试
    #[test]
    fn test_as_ref() {
        let obj = create_test_object();
        let map: &IndexMap<Utf8, NumberType> = obj.as_ref();
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_deref() {
        let obj = create_test_object();
        let map: &IndexMap<Utf8, NumberType> = &obj;
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_display() {
        let obj = create_test_object();
        let display = format!("{}", obj);
        // 顺序可能不同，所以检查两种可能
        let valid1 = r#"{"a": 1, "b": 2}"#;
        let valid2 = r#"{"b": 2, "a": 1}"#;
        assert!(display == valid1 || display == valid2);
    }

    #[test]
    fn test_into_iter() {
        let obj = create_test_object();
        let mut iter = obj.into_iter();

        // 顺序可能不同
        let (k1, v1) = iter.next().unwrap();
        let (k2, v2) = iter.next().unwrap();
        assert!(k1.as_ref() == "a" || k1.as_ref() == "b");
        assert!(k2.as_ref() == "a" || k2.as_ref() == "b");
        assert_ne!(k1, k2);
        assert_eq!(v1 + v2, NumberType::new(3.0)); // 1 + 2 = 3
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_default() {
        let obj: ObjectType<NumberType> = ObjectType::default();
        assert_eq!(obj.len(), 0);
    }

    // Test TryFrom<&[u8]> success path
    #[test]
    fn test_tryfrom_slice_success() {
        let obj = create_test_object();
        let data = obj.clone().marshall().unwrap();
        let from_slice = ObjectType::<NumberType>::try_from(data.as_slice()).unwrap();
        assert_eq!(obj, from_slice);
    }

    // Test TryFrom<&[u8]> with wrong type marker
    #[test]
    fn test_tryfrom_slice_wrong_marker() {
        let data = vec![TypeMarker::Null as u8, 0x00, 0x00, 0x09];
        let result = ObjectType::<NumberType>::try_from(data.as_slice());
        assert!(matches!(
            result,
            Err(AmfError::TypeMarkerValueMismatch { .. })
        ));
    }

    // Test Unmarshall buffer too small error
    #[test]
    fn test_unmarshall_buffer_too_small() {
        // Only marker, missing object-end bytes
        let data = &[TypeMarker::Object as u8];
        let result = ObjectType::<NumberType>::unmarshall(data);
        assert!(matches!(
            result,
            Err(AmfError::BufferTooSmall { want: _, got: 1 })
        ));
    }

    // Test Display for empty object
    #[test]
    fn test_display_empty() {
        let empty: ObjectType<NumberType> = ObjectType::default();
        assert_eq!(format!("{}", empty), "{}");
    }

    // Test IntoIterator for empty
    #[test]
    fn test_into_iter_empty() {
        let empty: ECMAArrayType<NumberType> = ECMAArrayType::default();
        let mut iter = empty.into_iter();
        assert!(iter.next().is_none());
    }

    // Test AsRef and Borrow for NestedType
    #[test]
    fn test_as_ref_and_borrow() {
        let obj = create_test_object();
        let as_ref_map: &IndexMap<Utf8, NumberType> = obj.as_ref();
        let borrow_map: &IndexMap<Utf8, NumberType> = obj.borrow();
        assert_eq!(as_ref_map.len(), 2);
        assert_eq!(borrow_map.len(), 2);
    }

    // Test ECMAArrayType length mismatch custom error message
    #[test]
    fn test_ecma_array_unmarshall_length_mismatch_message() {
        let mut data = create_test_ecma_array().marshall().unwrap();
        // Corrupt length field to mismatch
        data[1..5].copy_from_slice(&0u32.to_be_bytes());
        let result = ECMAArrayType::<NumberType>::unmarshall(&data);
        if let Err(AmfError::Custom(msg)) = result {
            assert!(msg.contains("Invalid properties length"));
        } else {
            panic!("Expected Custom error on length mismatch");
        }
    }
}
