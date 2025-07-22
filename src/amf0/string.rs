use crate::amf0::type_marker::TypeMarker;
use crate::amf0::utf8::AmfUtf8;
use crate::errors::AmfError;
use crate::traits::{Marshall, MarshallLength, Unmarshall};
use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AmfUtf8ValuedType<const LBW: usize, const TM: u8> {
    inner: AmfUtf8<LBW>,
}

impl<const LBW: usize, const TM: u8> AmfUtf8ValuedType<LBW, TM> {
    pub fn new(inner: AmfUtf8<LBW>) -> Self {
        Self { inner }
    }

    pub fn new_from_string(value: String) -> Result<Self, AmfError> {
        let inner = AmfUtf8::<LBW>::new(value)?;
        Ok(Self::new(inner))
    }

    pub fn new_from_str(value: &str) -> Result<Self, AmfError> {
        Self::new_from_string(value.to_string())
    }
}

impl<const LBW: usize, const TM: u8> Marshall for AmfUtf8ValuedType<LBW, TM> {
    fn marshall(&self) -> Result<Vec<u8>, AmfError> {
        let mut vec = Vec::with_capacity(self.marshall_length());
        vec.push(TM);
        let inner_vec = self.inner.marshall()?;
        vec.extend_from_slice(inner_vec.as_slice());
        Ok(vec)
    }
}

impl<const LBW: usize, const TM: u8> MarshallLength for AmfUtf8ValuedType<LBW, TM> {
    fn marshall_length(&self) -> usize {
        1 + self.inner.marshall_length()
    }
}

impl<const LBW: usize, const TM: u8> Unmarshall for AmfUtf8ValuedType<LBW, TM> {
    fn unmarshall(buf: &[u8]) -> Result<(Self, usize), AmfError> {
        let required_size = 1 + LBW;
        if buf.len() < required_size {
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
        let inner = AmfUtf8::unmarshall(&buf[1..])?;
        Ok((Self::new(inner.0), 1 + inner.1))
    }
}

// 实现 rust 惯用语("idiom") 方便用户使用

impl<const LBW: usize, const TM: u8> TryFrom<&[u8]> for AmfUtf8ValuedType<LBW, TM> {
    type Error = AmfError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::unmarshall(value).map(|(inner, _)| inner)
    }
}

impl<const LBW: usize, const TM: u8> TryFrom<Vec<u8>> for AmfUtf8ValuedType<LBW, TM> {
    type Error = AmfError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_slice())
    }
}

impl<const LBW: usize, const TM: u8> TryFrom<AmfUtf8ValuedType<LBW, TM>> for Vec<u8> {
    type Error = AmfError;

    fn try_from(value: AmfUtf8ValuedType<LBW, TM>) -> Result<Self, Self::Error> {
        value.marshall()
    }
}

impl<const LBW: usize, const TM: u8> TryFrom<String> for AmfUtf8ValuedType<LBW, TM> {
    type Error = AmfError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new_from_string(value)
    }
}

impl<const LBW: usize, const TM: u8> TryFrom<AmfUtf8ValuedType<LBW, TM>> for String {
    type Error = AmfError;

    fn try_from(value: AmfUtf8ValuedType<LBW, TM>) -> Result<Self, Self::Error> {
        value.inner.try_into()
    }
}

impl<const LBW: usize, const TM: u8> TryFrom<&str> for AmfUtf8ValuedType<LBW, TM> {
    type Error = AmfError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new_from_str(value)
    }
}

impl<const LBW: usize, const TM: u8> From<AmfUtf8<LBW>> for AmfUtf8ValuedType<LBW, TM> {
    fn from(value: AmfUtf8<LBW>) -> Self {
        Self::new(value)
    }
}

impl<const LBW: usize, const TM: u8> AsRef<AmfUtf8<LBW>> for AmfUtf8ValuedType<LBW, TM> {
    fn as_ref(&self) -> &AmfUtf8<LBW> {
        &self.inner
    }
}

impl<const LBW: usize, const TM: u8> Deref for AmfUtf8ValuedType<LBW, TM> {
    type Target = AmfUtf8<LBW>;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<const LBW: usize, const TM: u8> Borrow<AmfUtf8<LBW>> for AmfUtf8ValuedType<LBW, TM> {
    fn borrow(&self) -> &AmfUtf8<LBW> {
        self.as_ref()
    }
}

impl<const LBW: usize, const TM: u8> Display for AmfUtf8ValuedType<LBW, TM> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl<const LBW: usize, const TM: u8> Default for AmfUtf8ValuedType<LBW, TM> {
    fn default() -> Self {
        Self::new(AmfUtf8::<LBW>::default())
    }
}

// 类型别名

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
    use std::hash::{DefaultHasher, Hash, Hasher};

    // 测试 AmfUtf8ValuedType 的通用功能
    #[test]
    fn test_new() {
        let utf8 = AmfUtf8::<2>::new_from_str("test").unwrap();
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
        let utf8 = AmfUtf8::<2>::new_from_str("test").unwrap();
        let valued: AmfUtf8ValuedType<2, 0x02> = utf8.clone().try_into().unwrap();
        assert_eq!(valued.inner, utf8);
    }

    #[test]
    fn test_as_ref() {
        let utf8 = AmfUtf8::<2>::new_from_str("test").unwrap();
        let valued = AmfUtf8ValuedType::<2, 0x02>::new(utf8.clone());
        assert_eq!(valued.as_ref(), &utf8);
    }

    #[test]
    fn test_deref() {
        let utf8 = AmfUtf8::<2>::new_from_str("test").unwrap();
        let valued = AmfUtf8ValuedType::<2, 0x02>::new(utf8.clone());
        assert_eq!(&*valued, &utf8);
    }

    #[test]
    fn test_display() {
        let valued = AmfUtf8ValuedType::<2, 0x02>::new(AmfUtf8::<2>::new_from_str("test").unwrap());
        assert_eq!(format!("{}", valued), "test");
    }

    // 测试 StringType 具体实现
    #[test]
    fn test_string_type_marshall() {
        let s = StringType::new(AmfUtf8::<2>::new_from_str("hello").unwrap());
        let data = s.marshall().unwrap();
        assert_eq!(data[0], TypeMarker::String as u8);
        assert_eq!(&data[1..], [0x00, 0x05, b'h', b'e', b'l', b'l', b'o']);
    }

    #[test]
    fn test_string_type_marshall_length() {
        let s = StringType::new(AmfUtf8::<2>::new_from_str("hello").unwrap());
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
        let s = LongStringType::new(AmfUtf8::<4>::new_from_str("hello").unwrap());
        let data = s.marshall().unwrap();
        assert_eq!(data[0], TypeMarker::LongString as u8);
        assert_eq!(
            &data[1..],
            [0x00, 0x00, 0x00, 0x05, b'h', b'e', b'l', b'l', b'o']
        );
    }

    #[test]
    fn test_long_string_type_marshall_length() {
        let s = LongStringType::new(AmfUtf8::<4>::new_from_str("hello").unwrap());
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
        let s: StringType = AmfUtf8::<2>::new_from_str("test")
            .unwrap()
            .try_into()
            .unwrap();
        assert_eq!(s.as_ref().as_ref(), "test");
    }

    #[test]
    fn test_long_string_type_alias() {
        let s: LongStringType = AmfUtf8::<4>::new_from_str("test")
            .unwrap()
            .try_into()
            .unwrap();
        assert_eq!(s.as_ref().as_ref(), "test");
    }

    /// Helper to compute the hash of any `T: Hash`
    fn hash_of<T: Hash>(t: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        t.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn string_type_clone_and_eq() {
        // create an original value
        let inner = AmfUtf8::<2>::new_from_str("hello").unwrap();
        let orig: StringType = StringType::new(inner);

        // Clone should produce an equal value
        let cloned = orig.clone();
        assert_eq!(orig, cloned, "Clone must preserve value (PartialEq/ Eq)");

        // Hash of orig and cloned should be the same
        let h1 = hash_of(&orig);
        let h2 = hash_of(&cloned);
        assert_eq!(h1, h2, "Hash must be consistent for equal values");
    }

    #[test]
    fn string_type_hash_differs_on_content_change() {
        let a = StringType::new(AmfUtf8::<2>::new_from_str("foo").unwrap());
        let b = StringType::new(AmfUtf8::<2>::new_from_str("bar").unwrap());
        // different strings must produce different hashes (very likely)
        assert_ne!(
            hash_of(&a),
            hash_of(&b),
            "Different values should hash differently"
        );
    }

    #[test]
    fn long_string_type_clone_and_eq() {
        let inner = AmfUtf8::<4>::new_from_str("a very long string").unwrap();
        let orig: LongStringType = LongStringType::new(inner);

        // Clone ↔ Eq
        let cloned = orig.clone();
        assert_eq!(orig, cloned);

        // Hash consistency
        assert_eq!(hash_of(&orig), hash_of(&cloned));
    }

    #[test]
    fn long_string_type_hash_differs_on_content_change() {
        let a = LongStringType::new(AmfUtf8::<4>::new_from_str("one").unwrap());
        let b = LongStringType::new(AmfUtf8::<4>::new_from_str("two").unwrap());
        assert_ne!(hash_of(&a), hash_of(&b));
    }
    #[test]
    fn test_string_type_clone_partial_eq() {
        let s1: StringType = StringType::default();
        let s2 = s1.clone();
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_long_string_type_clone_partial_eq() {
        let ls1: LongStringType = LongStringType::default();
        let ls2 = ls1.clone();
        assert_eq!(ls1, ls2);
    }
}
