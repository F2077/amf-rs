use crate::traits::{FromBytes, ToBytes};
use crate::type_marker::TypeMarker;
use crate::utf8;
use crate::utf8::{AmfUtf8, Length, Utf8};
use indexmap::IndexMap;
use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter};
use std::io;
use std::ops::Deref;

pub trait AmfType: ToBytes + FromBytes {}

impl<T> AmfType for T where T: ToBytes + FromBytes + Display {}

// An AMF 0 Number type is used to encode an ActionScript Number.
// The data following a Number type marker is always an 8 byte IEEE-754 double precision floating point value in network byte order (sign bit in low memory).
#[derive(Debug, PartialEq)]
pub struct NumberType {
    type_marker: TypeMarker,
    value: f64,
}

impl NumberType {
    pub fn new(value: f64) -> Self {
        Self {
            type_marker: TypeMarker::Number,
            value,
        }
    }
}

impl ToBytes for NumberType {
    fn to_bytes(&self) -> io::Result<Vec<u8>> {
        debug_assert!(self.type_marker == TypeMarker::Number);
        let mut vec = Vec::with_capacity(self.bytes_size());
        vec.push(self.type_marker as u8);
        vec.extend_from_slice(&self.value.to_be_bytes());
        Ok(vec)
    }

    fn bytes_size(&self) -> usize {
        1 + 8 // 1 byte for type marker + 8 bytes for value
    }

    fn write_bytes_to(&self, buf: &mut [u8]) -> io::Result<usize> {
        debug_assert!(self.type_marker == TypeMarker::Number);
        let required_size = self.bytes_size();
        if buf.len() < required_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Buffer is too small, need at least {} bytes", required_size),
            ));
        }

        buf[0] = self.type_marker as u8;
        buf[1..9].copy_from_slice(&self.value.to_be_bytes());
        Ok(9)
    }
}

impl FromBytes for NumberType {
    fn from_bytes(buf: &[u8]) -> io::Result<(Self, usize)> {
        if buf.len() < 9 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Buffer is too small, need at least 9 bytes",
            ));
        }
        let type_marker = TypeMarker::try_from(buf[0])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if type_marker != TypeMarker::Number {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid type marker, expected Number, got {:?}",
                    type_marker
                ),
            ));
        }
        let value = f64::from_be_bytes(buf[1..9].try_into().unwrap());
        Ok((Self { type_marker, value }, 9))
    }
}

impl Deref for NumberType {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl AsRef<f64> for NumberType {
    fn as_ref(&self) -> &f64 {
        &self.value
    }
}

impl Borrow<f64> for NumberType {
    fn borrow(&self) -> &f64 {
        &self.value
    }
}

impl Display for NumberType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl TryFrom<f64> for NumberType {
    type Error = io::Error;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Ok(NumberType::new(value))
    }
}

//	An AMF 0 Boolean type is used to encode a primitive ActionScript 1.0 or 2.0 Boolean or
//	an ActionScript 3.0 Boolean. The Object (non-primitive) version of ActionScript 1.0 or
//	2.0 Booleans are not serializable. A Boolean type marker is followed by an unsigned
//	byte; a zero byte value denotes false while a non-zero byte value (typically 1) denotes
//	true.
#[derive(Debug, PartialEq)]
pub struct BooleanType {
    type_marker: TypeMarker,
    value: bool,
}

impl BooleanType {
    pub fn new(value: bool) -> Self {
        Self {
            type_marker: TypeMarker::Boolean,
            value,
        }
    }
}

impl ToBytes for BooleanType {
    fn to_bytes(&self) -> io::Result<Vec<u8>> {
        debug_assert!(self.type_marker == TypeMarker::Boolean);
        let mut vec = Vec::with_capacity(self.bytes_size());
        vec.push(self.type_marker as u8);
        vec.push(self.value as u8);
        Ok(vec)
    }

    fn bytes_size(&self) -> usize {
        1 + 1 // 1 byte for type marker + 1 byte for value
    }

    fn write_bytes_to(&self, buf: &mut [u8]) -> io::Result<usize> {
        debug_assert!(self.type_marker == TypeMarker::Boolean);
        let required_size = self.bytes_size();
        if buf.len() < required_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Buffer is too small, need at least {} bytes", required_size),
            ));
        }
        buf[0] = self.type_marker as u8;
        buf[1] = self.value as u8;
        Ok(2)
    }
}

impl FromBytes for BooleanType {
    fn from_bytes(buf: &[u8]) -> io::Result<(Self, usize)> {
        if buf.len() < 2 {
            return Err(io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Buffer is too small, need at least 2 bytes",
            ));
        }
        let type_marker = TypeMarker::try_from(buf[0])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if type_marker != TypeMarker::Boolean {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid type marker, expected Boolean, got {:?}",
                    type_marker
                ),
            ));
        }
        let value = buf[1] != 0;
        Ok((Self { type_marker, value }, 2))
    }
}

impl Deref for BooleanType {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl AsRef<bool> for BooleanType {
    fn as_ref(&self) -> &bool {
        &self.value
    }
}

impl Borrow<bool> for BooleanType {
    fn borrow(&self) -> &bool {
        &self.value
    }
}

impl Display for BooleanType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl TryFrom<bool> for BooleanType {
    type Error = io::Error;

    fn try_from(value: bool) -> Result<Self, Self::Error> {
        Ok(BooleanType::new(value))
    }
}

#[derive(Debug, PartialEq)]
pub struct AmfUtf8ValuedType<'a, L: Length, const M: u8> {
    inner: AmfUtf8<'a, L>,
}

impl<'a, L: Length + TryInto<usize>, const M: u8> AmfUtf8ValuedType<'a, L, M> {
    pub fn new(inner: AmfUtf8<'a, L>) -> Self {
        Self { inner }
    }
}

impl<'a, L: Length + TryInto<usize>, const M: u8> ToBytes for AmfUtf8ValuedType<'a, L, M> {
    fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut vec = Vec::with_capacity(self.bytes_size());
        vec.push(M);
        let inner_vec = self.inner.to_bytes()?;
        vec.extend_from_slice(inner_vec.as_slice());
        Ok(vec)
    }

    fn bytes_size(&self) -> usize {
        1 + self.inner.bytes_size()
    }

    fn write_bytes_to(&self, buf: &mut [u8]) -> io::Result<usize> {
        let required_size = self.bytes_size();
        if buf.len() < required_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Buffer is too small, need at least {} bytes", required_size),
            ));
        }
        buf[0] = M;
        let n = self.inner.write_bytes_to(&mut buf[1..])?;
        Ok(1 + n)
    }
}

impl<'a, L: Length + TryInto<usize>, const M: u8> FromBytes for AmfUtf8ValuedType<'a, L, M> {
    fn from_bytes(buf: &[u8]) -> io::Result<(Self, usize)> {
        let required_size = 1 + L::WIDTH;
        if buf.len() < required_size {
            return Err(io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Buffer is too small, need at least {} bytes", required_size),
            ));
        }

        let type_marker = TypeMarker::try_from(buf[0])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if buf[0] != M {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid type marker, expected String, got {:?}",
                    type_marker
                ),
            ));
        }
        let inner = AmfUtf8::from_bytes(&buf[1..])?;
        Ok((Self::new(inner.0), 1 + inner.1))
    }
}

impl<'a, L: Length + TryInto<usize>, const M: u8> Deref for AmfUtf8ValuedType<'a, L, M> {
    type Target = AmfUtf8<'a, L>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, L: Length + TryInto<usize>, const M: u8> AsRef<AmfUtf8<'a, L>>
    for AmfUtf8ValuedType<'a, L, M>
{
    fn as_ref(&self) -> &AmfUtf8<'a, L> {
        &self.inner
    }
}

impl<'a, L: Length + TryInto<usize>, const M: u8> Borrow<AmfUtf8<'a, L>>
    for AmfUtf8ValuedType<'a, L, M>
{
    fn borrow(&self) -> &AmfUtf8<'a, L> {
        &self.inner
    }
}

impl<'a, L: Length + TryInto<usize>, const M: u8> Display for AmfUtf8ValuedType<'a, L, M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl<'a, L: Length + TryInto<usize>, const M: u8> TryFrom<AmfUtf8<'a, L>>
    for AmfUtf8ValuedType<'a, L, M>
{
    type Error = io::Error;

    fn try_from(value: AmfUtf8<'a, L>) -> Result<Self, Self::Error> {
        Ok(Self::new(value))
    }
}

//	All strings in AMF are encoded using UTF-8; however, the byte-length header format
//	may vary. The AMF 0 String type uses the standard byte-length header (i.e. U16). For
//	long Strings that require more than 65535 bytes to encode in UTF-8, the AMF 0 Long
//	String type should be used.
pub type StringType<'a> = AmfUtf8ValuedType<'a, u16, { TypeMarker::String as u8 }>;

//	A long string is used in AMF 0 to encode strings that would occupy more than 65535
//	bytes when UTF-8 encoded. The byte-length header of the UTF-8 encoded string is a 32-
//	bit integer instead of the regular 16-bit integer.
pub type LongStringType<'a> = AmfUtf8ValuedType<'a, u32, { TypeMarker::LongString as u8 }>;

#[derive(Debug, PartialEq)]
pub struct ObjectEndType {
    empty: Utf8<'static>,
    type_marker: TypeMarker,
}

impl ObjectEndType {
    pub fn new() -> Self {
        Self {
            empty: utf8::EMPTY_UTF8,
            type_marker: TypeMarker::ObjectEnd,
        }
    }
}

impl ToBytes for ObjectEndType {
    fn to_bytes(&self) -> io::Result<Vec<u8>> {
        debug_assert!(self.type_marker == TypeMarker::ObjectEnd);
        let mut vec = Vec::with_capacity(self.bytes_size());
        vec.extend_from_slice(&self.empty.to_bytes()?);
        vec.push(self.type_marker as u8);
        Ok(vec)
    }

    fn bytes_size(&self) -> usize {
        2 + 1
    }

    fn write_bytes_to(&self, buf: &mut [u8]) -> io::Result<usize> {
        debug_assert!(self.type_marker == TypeMarker::ObjectEnd);
        let required_size = self.bytes_size();
        if buf.len() < required_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Buffer is too small, need at least {} bytes", required_size),
            ));
        }
        let _ = self.empty.write_bytes_to(&mut buf[0..2])?;
        buf[2] = self.type_marker as u8;
        Ok(3)
    }
}

impl FromBytes for ObjectEndType {
    fn from_bytes(buf: &[u8]) -> io::Result<(Self, usize)> {
        if buf.len() < 3 {
            return Err(io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Buffer is too small, need at least 3 bytes",
            ));
        }
        let (empty, _) = Utf8::from_bytes(&buf[0..2])?;
        let type_marker = TypeMarker::try_from(buf[2])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if type_marker != TypeMarker::ObjectEnd {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid type marker, expected ObjectEnd, got {:?}",
                    type_marker
                ),
            ));
        }
        Ok((Self { empty, type_marker }, 3))
    }
}

impl Deref for ObjectEndType {
    type Target = Utf8<'static>;

    fn deref(&self) -> &Self::Target {
        &self.empty
    }
}

impl AsRef<Utf8<'static>> for ObjectEndType {
    fn as_ref(&self) -> &Utf8<'static> {
        &self.empty
    }
}

impl Borrow<Utf8<'static>> for ObjectEndType {
    fn borrow(&self) -> &Utf8<'static> {
        &self.empty
    }
}

impl Display for ObjectEndType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.empty)
    }
}

impl TryFrom<Utf8<'static>> for ObjectEndType {
    type Error = io::Error;

    fn try_from(value: Utf8<'static>) -> Result<Self, Self::Error> {
        if value == utf8::EMPTY_UTF8 {
            Ok(Self::new())
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid ObjectEndType",
            ))
        }
    }
}

pub const OBJECT_END: ObjectEndType = ObjectEndType {
    empty: utf8::EMPTY_UTF8,
    type_marker: TypeMarker::ObjectEnd,
};

#[derive(Debug, PartialEq)]
pub struct NestedType<'a, T: AmfType, const M: u8, const W: usize> {
    length: Option<u32>,
    properties: IndexMap<Utf8<'a>, T>,
    object_end: ObjectEndType,
}

impl<'a, T: AmfType, const M: u8, const W: usize> NestedType<'a, T, M, W> {
    pub fn new(properties: IndexMap<Utf8<'a>, T>) -> Self {
        let length = if W == 4 {
            Some(properties.len() as u32)
        } else {
            None
        };
        Self {
            length,
            properties,
            object_end: OBJECT_END,
        }
    }
}

impl<'a, T: AmfType, const M: u8, const W: usize> ToBytes for NestedType<'a, T, M, W> {
    fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut vec = Vec::with_capacity(self.bytes_size());
        vec.push(M);

        if let Some(length) = self.length {
            let length_bytes = length.to_be_bytes();
            vec.extend_from_slice(&length_bytes);
        }

        self.properties
            .iter()
            .try_for_each(|(k, v)| -> io::Result<()> {
                let k_vec = k
                    .to_bytes()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                vec.extend_from_slice(&k_vec);
                let v_vec = v
                    .to_bytes()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                vec.extend_from_slice(&v_vec);
                Ok(())
            })?;

        let object_end_vec = self.object_end.to_bytes()?;
        vec.extend_from_slice(&object_end_vec);

        Ok(vec)
    }

    fn bytes_size(&self) -> usize {
        let mut size = 1; // 1 byte for type marker
        size += W;
        let properties_bytes_size: usize = self
            .properties
            .iter()
            .map(|(k, v)| k.bytes_size() + v.bytes_size())
            .sum();
        size += properties_bytes_size;
        size += self.object_end.bytes_size();
        size
    }

    fn write_bytes_to(&self, buf: &mut [u8]) -> io::Result<usize> {
        let required_size = self.bytes_size();
        if buf.len() < required_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Buffer is too small, need at least {} bytes", required_size),
            ));
        }
        buf[0] = M;
        if let Some(length) = self.length {
            let length_bytes = length.to_be_bytes();
            buf[1..1 + W].copy_from_slice(&length_bytes);
        }
        let mut offset = 1 + W;
        for (k, v) in &self.properties {
            k.write_bytes_to(&mut buf[offset..offset + k.bytes_size()])?;
            offset += k.bytes_size();
            v.write_bytes_to(&mut buf[offset..offset + v.bytes_size()])?;
            offset += v.bytes_size();
        }
        Ok(offset)
    }
}

impl<'a, T: AmfType, const M: u8, const W: usize> FromBytes for NestedType<'a, T, M, W> {
    fn from_bytes(buf: &[u8]) -> io::Result<(Self, usize)> {
        let required_size = 1 + W + 3;
        if buf.len() < required_size {
            // 1 byte for type marker, W bytes(maybe 0) for optional properties length,  3 bytes for object end
            return Err(io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Buffer is too small, need at least {} bytes", required_size),
            ));
        }

        let type_marker = TypeMarker::try_from(buf[0])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if buf[0] != M {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid type marker, expected {}, got {:?}", M, type_marker),
            ));
        }

        let mut length = 0u32;
        if W == 4 {
            length = u32::from_be_bytes(
                buf[1..1 + W]
                    .try_into()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            );
        }

        let mut properties = IndexMap::new();
        let mut offset = 1 + W;
        loop {
            if offset == buf.len() - 3 {
                if buf[offset] == OBJECT_END.type_marker as u8 {
                    break;
                } else {
                    return Err(io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid object end marker",
                    ));
                }
            }

            let (k, k_len) = Utf8::from_bytes(&buf[offset..])?;
            offset += k_len;
            let (v, v_len) = T::from_bytes(&buf[offset..])?;
            offset += v_len;
            properties.insert(k, v);
        }

        if properties.len() != length as usize {
            return Err(io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Invalid properties length, expected {}, got {}",
                    properties.len(),
                    length
                ),
            ));
        }

        Ok((Self::new(properties), offset))
    }
}

impl<'a, T: AmfType, const M: u8, const W: usize> Deref for NestedType<'a, T, M, W> {
    type Target = IndexMap<Utf8<'a>, T>;

    fn deref(&self) -> &Self::Target {
        &self.properties
    }
}

impl<'a, T: AmfType, const M: u8, const W: usize> AsRef<IndexMap<Utf8<'a>, T>>
    for NestedType<'a, T, M, W>
{
    fn as_ref(&self) -> &IndexMap<Utf8<'a>, T> {
        &self.properties
    }
}

impl<'a, T: AmfType, const M: u8, const W: usize> Display for NestedType<'a, T, M, W> {
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

//	The AMF 0 Object type is used to encoded anonymous ActionScript objects. Any typed
//	object that does not have a registered class should be treated as an anonymous
//	ActionScript object. If the same object instance appears in an object graph it should be
//	sent by reference using an AMF 0.
//	Use the reference type to reduce redundant information from being serialized and infinite
//	loops from cyclical references.
pub type ObjectType<'a, T: AmfType> = NestedType<'a, T, { TypeMarker::Object as u8 }, 0>;

// An ECMA Array or 'associative' Array is used when an ActionScript Array contains non-ordinal indices.
// This type is considered a complex type and thus reoccurring instancescan be sent by reference.
// All indices. ordinal or otherwise, are treated as string keysinstead of integers.
// For the purposes of serialization this type is very similar to ananonymous Obiect.
pub type ECMAArrayType<'a, T: AmfType> = NestedType<'a, T, { TypeMarker::EcmaArray as u8 }, 4>;

trait MarkerType: Sized {
    const TYPE_MARKER: TypeMarker;
}

impl<M: MarkerType> ToBytes for M {
    fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut vec = Vec::with_capacity(self.bytes_size());
        vec.push(M::TYPE_MARKER as u8);
        Ok(vec)
    }

    fn bytes_size(&self) -> usize {
        1
    }

    fn write_bytes_to(&self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.len() < 1 {
            return Err(io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Buffer is too small, need at least 1 byte",
            ));
        }
        buf[0] = M::TYPE_MARKER as u8;
        Ok(1)
    }
}

impl<M: MarkerType + Default> FromBytes for M {
    fn from_bytes(buf: &[u8]) -> io::Result<(Self, usize)> {
        if buf.len() < 1 {
            return Err(io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Buffer is too small, need at least 1 byte",
            ));
        }
        let type_marker = TypeMarker::try_from(buf[0])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if type_marker != M::TYPE_MARKER {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid type marker, expected {:?}, got {:?}",
                    M::TYPE_MARKER,
                    type_marker
                ),
            ));
        }
        Ok((M::default(), 1))
    }
}

//	The null type is represented by the null type marker. No further information is encoded
//	for this value.
#[derive(Debug, PartialEq, Default)]
pub struct NullType;

impl MarkerType for NullType {
    const TYPE_MARKER: TypeMarker = TypeMarker::Null;
}

impl Display for NullType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "null")
    }
}

//    The undefined type is represented by the undefined type marker. No further information is encoded
//    for this value.
#[derive(Debug, PartialEq, Default)]
pub struct UndefinedType;

impl MarkerType for UndefinedType {
    const TYPE_MARKER: TypeMarker = TypeMarker::Undefined;
}

impl Display for UndefinedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "undefined")
    }
}

pub struct DoNotSupportType {}

impl ToBytes for DoNotSupportType {
    fn to_bytes(&self) -> io::Result<Vec<u8>> {
        panic!("Do not support")
    }

    fn bytes_size(&self) -> usize {
        panic!("Do not support")
    }

    fn write_bytes_to(&self, buf: &mut [u8]) -> io::Result<usize> {
        panic!("Do not support")
    }
}

impl FromBytes for DoNotSupportType {
    fn from_bytes(buf: &[u8]) -> io::Result<(Self, usize)> {
        panic!("Do not support")
    }
}

//	If a type cannot be serialized a special unsupported marker can be used in place of the
//	type. Some endpoints may throw an error on encountering this type marker. No further
//	information is encoded for this type.
pub type UnsupportedType = DoNotSupportType;

//	This type is not supported and is reserved for future use.
pub type MovieClipType = DoNotSupportType;

//	This type is not supported and is reserved for future use.
pub type RecordSetType = DoNotSupportType;

pub type ReferenceType = DoNotSupportType;
pub type StrictArrayType = DoNotSupportType;
pub type DateType = DoNotSupportType;
pub type XMLDocumentType = DoNotSupportType;
pub type TypedObjectType = DoNotSupportType;
