use crate::amf0::type_marker::TypeMarker;
use crate::amf0::utf8::AmfUtf8;
use crate::errors::AmfError;
use crate::traits::{Marshall, MarshallLength, Unmarshall};
use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::io;
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq)]
pub struct AmfUtf8ValuedType<const LENGTH_BYTE_WIDTH: usize, const TYPE_MARKER: u8> {
    inner: AmfUtf8<LENGTH_BYTE_WIDTH>,
}

impl<const LENGTH_BYTE_WIDTH: usize, const TYPE_MARKER: u8>
    AmfUtf8ValuedType<LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    pub fn new(inner: AmfUtf8<LENGTH_BYTE_WIDTH>) -> Self {
        Self { inner }
    }
}

impl<const LENGTH_BYTE_WIDTH: usize, const TYPE_MARKER: u8> Marshall
    for AmfUtf8ValuedType<LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    fn marshall(&self) -> Result<Vec<u8>, AmfError> {
        let mut vec = Vec::with_capacity(self.marshall_length());
        vec.push(TYPE_MARKER);
        let inner_vec = self.inner.marshall()?;
        vec.extend_from_slice(inner_vec.as_slice());
        Ok(vec)
    }
}

impl<const LENGTH_BYTE_WIDTH: usize, const TYPE_MARKER: u8> MarshallLength
    for AmfUtf8ValuedType<LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    fn marshall_length(&self) -> usize {
        1 + self.inner.marshall_length()
    }
}

impl<const LENGTH_BYTE_WIDTH: usize, const TYPE_MARKER: u8> Unmarshall
    for AmfUtf8ValuedType<LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    fn unmarshall(buf: &[u8]) -> Result<(Self, usize), AmfError> {
        let required_size = 1 + LENGTH_BYTE_WIDTH;
        if buf.len() < required_size {
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
        let inner = AmfUtf8::unmarshall(&buf[1..])?;
        Ok((Self::new(inner.0), 1 + inner.1))
    }
}

impl<const LENGTH_BYTE_WIDTH: usize, const TYPE_MARKER: u8> TryFrom<AmfUtf8<LENGTH_BYTE_WIDTH>>
    for AmfUtf8ValuedType<LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    type Error = io::Error;

    fn try_from(value: AmfUtf8<LENGTH_BYTE_WIDTH>) -> Result<Self, Self::Error> {
        Ok(Self::new(value))
    }
}

impl<const LENGTH_BYTE_WIDTH: usize, const TYPE_MARKER: u8> AsRef<AmfUtf8<LENGTH_BYTE_WIDTH>>
    for AmfUtf8ValuedType<LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    fn as_ref(&self) -> &AmfUtf8<LENGTH_BYTE_WIDTH> {
        &self.inner
    }
}

impl<const LENGTH_BYTE_WIDTH: usize, const TYPE_MARKER: u8> Deref
    for AmfUtf8ValuedType<LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    type Target = AmfUtf8<LENGTH_BYTE_WIDTH>;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<const LENGTH_BYTE_WIDTH: usize, const TYPE_MARKER: u8> Borrow<AmfUtf8<LENGTH_BYTE_WIDTH>>
    for AmfUtf8ValuedType<LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    fn borrow(&self) -> &AmfUtf8<LENGTH_BYTE_WIDTH> {
        self.as_ref()
    }
}

impl<const LENGTH_BYTE_WIDTH: usize, const TYPE_MARKER: u8> Display
    for AmfUtf8ValuedType<LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl<const LENGTH_BYTE_WIDTH: usize, const TYPE_MARKER: u8> Default
    for AmfUtf8ValuedType<LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    fn default() -> Self {
        Self::new(AmfUtf8::<LENGTH_BYTE_WIDTH>::default())
    }
}

//	All strings in AMF are encoded using UTF-8; however, the byte-length header format
//	may vary. The AMF 0 String type uses the standard byte-length header (i.e. U16). For
//	long Strings that require more than 65535 bytes to encode in UTF-8, the AMF 0 Long
//	String type should be used.
pub type StringType = AmfUtf8ValuedType<2, { TypeMarker::String as u8 }>;

//	A long string is used in AMF 0 to encode strings that would occupy more than 65535
//	bytes when UTF-8 encoded. The byte-length header of the UTF-8 encoded string is a 32-
//	bit integer instead of the regular 16-bit integer.
pub type LongStringType = AmfUtf8ValuedType<4, { TypeMarker::LongString as u8 }>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amf0::utf8::AmfUtf8;

    // 测试 AmfUtf8ValuedType 的通用功能
    #[test]
    fn test_new() {
        let utf8 = AmfUtf8::<2>::new("test").unwrap();
        let valued = AmfUtf8ValuedType::<2, 0x02>::new(utf8.clone());
        assert_eq!(valued.inner, utf8);
    }

    #[test]
    fn test_default() {
        let valued = AmfUtf8ValuedType::<2, 0x02>::default();
        assert_eq!(valued.inner, AmfUtf8::<2>::default());
    }

    #[test]
    fn test_try_from() {
        let utf8 = AmfUtf8::<2>::new("test").unwrap();
        let valued: AmfUtf8ValuedType<2, 0x02> = utf8.clone().try_into().unwrap();
        assert_eq!(valued.inner, utf8);
    }

    #[test]
    fn test_as_ref() {
        let utf8 = AmfUtf8::<2>::new("test").unwrap();
        let valued = AmfUtf8ValuedType::<2, 0x02>::new(utf8.clone());
        assert_eq!(valued.as_ref(), &utf8);
    }

    #[test]
    fn test_deref() {
        let utf8 = AmfUtf8::<2>::new("test").unwrap();
        let valued = AmfUtf8ValuedType::<2, 0x02>::new(utf8.clone());
        assert_eq!(&*valued, &utf8);
    }

    #[test]
    fn test_display() {
        let valued = AmfUtf8ValuedType::<2, 0x02>::new(AmfUtf8::<2>::new("test").unwrap());
        assert_eq!(format!("{}", valued), "test");
    }

    // 测试 StringType 具体实现
    #[test]
    fn test_string_type_marshall() {
        let s = StringType::new(AmfUtf8::<2>::new("hello").unwrap());
        let data = s.marshall().unwrap();
        assert_eq!(data[0], TypeMarker::String as u8);
        assert_eq!(&data[1..], [0x00, 0x05, b'h', b'e', b'l', b'l', b'o']);
    }

    #[test]
    fn test_string_type_marshall_length() {
        let s = StringType::new(AmfUtf8::<2>::new("hello").unwrap());
        assert_eq!(s.marshall_length(), 8); // 1 marker + 2 length + 5 chars
    }

    #[test]
    fn test_string_type_unmarshall() {
        let data = [
            TypeMarker::String as u8,
            0x00,
            0x05, // length 5
            b'h',
            b'e',
            b'l',
            b'l',
            b'o',
        ];
        let (s, bytes_read) = StringType::unmarshall(&data).unwrap();
        assert_eq!(bytes_read, 8);
        assert_eq!(s.as_ref().as_ref(), "hello");
    }

    #[test]
    fn test_string_type_unmarshall_invalid_marker() {
        let data = [
            TypeMarker::Number as u8, // wrong marker
            0x00,
            0x05,
            b'h',
            b'e',
            b'l',
            b'l',
            b'o',
        ];
        let result = StringType::unmarshall(&data);
        assert!(matches!(
            result,
            Err(AmfError::TypeMarkerValueMismatch {
                want: 0x02,
                got: 0x00
            })
        ));
    }

    #[test]
    fn test_string_type_unmarshall_buffer_too_small() {
        let data = [TypeMarker::String as u8, 0x00]; // incomplete
        let result = StringType::unmarshall(&data);
        assert!(matches!(
            result,
            Err(AmfError::BufferTooSmall {
                want: 3, // marker + 2-byte length
                got: 2
            })
        ));
    }

    // 测试 LongStringType 具体实现
    #[test]
    fn test_long_string_type_marshall() {
        let s = LongStringType::new(AmfUtf8::<4>::new("hello").unwrap());
        let data = s.marshall().unwrap();
        assert_eq!(data[0], TypeMarker::LongString as u8);
        assert_eq!(
            &data[1..],
            [0x00, 0x00, 0x00, 0x05, b'h', b'e', b'l', b'l', b'o']
        );
    }

    #[test]
    fn test_long_string_type_marshall_length() {
        let s = LongStringType::new(AmfUtf8::<4>::new("hello").unwrap());
        assert_eq!(s.marshall_length(), 10); // 1 marker + 4 length + 5 chars
    }

    #[test]
    fn test_long_string_type_unmarshall() {
        let data = [
            TypeMarker::LongString as u8,
            0x00,
            0x00,
            0x00,
            0x05, // length 5
            b'h',
            b'e',
            b'l',
            b'l',
            b'o',
        ];
        let (s, bytes_read) = LongStringType::unmarshall(&data).unwrap();
        assert_eq!(bytes_read, 10);
        assert_eq!(s.as_ref().as_ref(), "hello");
    }

    #[test]
    fn test_long_string_type_unmarshall_large_string() {
        let long_str = "a".repeat(70_000);
        let mut data = vec![TypeMarker::LongString as u8];
        let len_bytes = (long_str.len() as u32).to_be_bytes();
        data.extend_from_slice(&len_bytes);
        data.extend_from_slice(long_str.as_bytes());

        let (s, bytes_read) = LongStringType::unmarshall(&data).unwrap();
        assert_eq!(bytes_read, 1 + 4 + long_str.len());
        assert_eq!(s.as_ref().as_ref(), long_str);
    }

    #[test]
    fn test_long_string_type_unmarshall_invalid_marker() {
        let data = [
            TypeMarker::String as u8, // wrong marker
            0x00,
            0x00,
            0x00,
            0x05,
            b'h',
            b'e',
            b'l',
            b'l',
            b'o',
        ];
        let result = LongStringType::unmarshall(&data);
        assert!(matches!(
            result,
            Err(AmfError::TypeMarkerValueMismatch {
                want: 0x0C,
                got: 0x02
            })
        ));
    }

    #[test]
    fn test_long_string_type_unmarshall_buffer_too_small() {
        let data = [TypeMarker::LongString as u8, 0x00, 0x00, 0x00]; // incomplete
        let result = LongStringType::unmarshall(&data);
        assert!(matches!(
            result,
            Err(AmfError::BufferTooSmall {
                want: 5, // marker + 4-byte length
                got: 4
            })
        ));
    }

    // 测试类型别名
    #[test]
    fn test_string_type_alias() {
        let s: StringType = AmfUtf8::<2>::new("test").unwrap().try_into().unwrap();
        assert_eq!(s.as_ref().as_ref(), "test");
    }

    #[test]
    fn test_long_string_type_alias() {
        let s: LongStringType = AmfUtf8::<4>::new("test").unwrap().try_into().unwrap();
        assert_eq!(s.as_ref().as_ref(), "test");
    }
}
