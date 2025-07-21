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

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NestedType<
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
        loop {
            if offset == buf.len() - 3 {
                // 找到了 object end 则退出循环
                let (object_end, _) = ObjectEndType::unmarshall(&buf[offset..])?;
                if object_end == ObjectEndType::default() {
                    break;
                }
            }
            if offset == buf.len() {
                return Err(AmfError::Custom(
                    "Invalid object, expected object end, got end of buffer".to_string(),
                ));
            }

            let (k, k_len) = Utf8::unmarshall(&buf[offset..])?;
            offset += k_len;
            let (v, v_len) = T::unmarshall(&buf[offset..])?;
            offset += v_len;
            properties.insert(k, v);
        }

        if properties.len() != length as usize {
            return Err(AmfError::Custom(format!(
                "Invalid properties length, want {}, got {}",
                length,
                properties.len()
            )));
        }

        Ok((Self::new(properties), offset))
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
