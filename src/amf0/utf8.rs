use crate::errors::AmfError;
use crate::traits::{Marshall, MarshallLength, Unmarshall};
use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AmfUtf8<const LBW: usize> {
    inner: String,
}

impl<const LBW: usize> AmfUtf8<LBW> {
    pub fn new(inner: String) -> Result<Self, AmfError> {
        debug_assert!(LBW == 2 || LBW == 4);
        let len = inner.len();
        if (LBW == 2 && len > u16::MAX as usize) || (LBW == 4 && len > u32::MAX as usize) {
            return Err(AmfError::StringTooLong { max: LBW, got: len });
        }
        Ok(Self {
            inner: inner.to_string(),
        })
    }

    pub fn new_from_str(inner: &str) -> Result<Self, AmfError> {
        Self::new(inner.to_string())
    }
}

impl<const LBW: usize> Marshall for AmfUtf8<LBW> {
    fn marshall(&self) -> Result<Vec<u8>, AmfError> {
        debug_assert!(LBW == 2 || LBW == 4);
        let mut vec = Vec::with_capacity(self.marshall_length());
        if LBW == 2 {
            vec.extend_from_slice((self.inner.len() as u16).to_be_bytes().as_slice())
        } else if LBW == 4 {
            vec.extend_from_slice((self.inner.len() as u32).to_be_bytes().as_slice())
        } else {
            return Err(AmfError::Custom("Invalid length byte width".to_string()));
        }
        vec.extend_from_slice(self.inner.as_bytes());
        Ok(vec)
    }
}

impl<const LBW: usize> MarshallLength for AmfUtf8<LBW> {
    fn marshall_length(&self) -> usize {
        debug_assert!(LBW == 2 || LBW == 4);
        LBW + self.inner.len()
    }
}

impl<const LBW: usize> Unmarshall for AmfUtf8<LBW> {
    fn unmarshall(buf: &[u8]) -> Result<(Self, usize), AmfError> {
        debug_assert!(LBW == 2 || LBW == 4);
        let length;
        if LBW == 2 {
            if buf.len() < 2 {
                return Err(AmfError::BufferTooSmall {
                    want: 2,
                    got: buf.len(),
                });
            }
            length = u16::from_be_bytes(buf[0..2].try_into().unwrap()) as usize;
        } else if LBW == 4 {
            if buf.len() < 4 {
                return Err(AmfError::BufferTooSmall {
                    want: 4,
                    got: buf.len(),
                });
            }
            length = u32::from_be_bytes(buf[0..4].try_into().unwrap()) as usize;
        } else {
            return Err(AmfError::Custom("Invalid length byte width".to_string()));
        }

        let start = LBW;
        let end = start + length;
        if buf.len() < end {
            return Err(AmfError::BufferTooSmall {
                want: end,
                got: buf.len(),
            });
        }
        let value = std::str::from_utf8(&buf[start..end]).map_err(|e| AmfError::InvalidUtf8(e))?;
        Ok((
            Self {
                inner: value.to_string(),
            },
            end,
        ))
    }
}

// 实现 rust 惯用语("idiom") 方便用户使用

impl<const LBW: usize> TryFrom<&[u8]> for AmfUtf8<LBW> {
    type Error = AmfError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::unmarshall(value).map(|(v, _)| v)
    }
}

impl<const LBW: usize> TryFrom<String> for AmfUtf8<LBW> {
    type Error = AmfError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<const LBW: usize> TryFrom<&str> for AmfUtf8<LBW> {
    type Error = AmfError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new_from_str(value)
    }
}

impl<const LBW: usize> AsRef<str> for AmfUtf8<LBW> {
    fn as_ref(&self) -> &str {
        self.inner.as_ref()
    }
}
impl<const LBW: usize> Deref for AmfUtf8<LBW> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        Self::as_ref(self)
    }
}
impl<const LBW: usize> Borrow<str> for AmfUtf8<LBW> {
    fn borrow(&self) -> &str {
        Self::as_ref(self)
    }
}

impl<const LBW: usize> Display for AmfUtf8<LBW> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl<const LBW: usize> Default for AmfUtf8<LBW> {
    fn default() -> Self {
        Self::new_from_str("").unwrap()
    }
}

// 类型别名

pub type Utf8 = AmfUtf8<2>;
pub type Utf8Long = AmfUtf8<4>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{Marshall, MarshallLength, Unmarshall};
    use std::hash::{DefaultHasher, Hash, Hasher};

    // 测试有效字符串创建（LBW=2）
    #[test]
    fn new_valid_utf8_w2() {
        let s = "a".repeat(u16::MAX as usize);
        let amf_str = AmfUtf8::<2>::new_from_str(&s).unwrap();
        assert_eq!(amf_str.inner, s);
    }

    // 测试过长字符串创建（LBW=2）
    #[test]
    fn new_too_long_utf8_w2() {
        let s = "a".repeat(u16::MAX as usize + 1);
        assert!(matches!(
            AmfUtf8::<2>::new_from_str(&s),
            Err(AmfError::StringTooLong { max: 2, got: _ })
        ));
    }

    // 测试有效字符串创建（LBW=4）
    #[test]
    fn new_valid_utf8_w4() {
        let s = "a".repeat(1000); // 在u32范围内
        let amf_str = AmfUtf8::<4>::new_from_str(&s).unwrap();
        assert_eq!(amf_str.inner, s);
    }

    // 测试序列化（LBW=2）
    #[test]
    fn try_into_bytes_w2() {
        let amf_str = AmfUtf8::<2>::new_from_str("hello").unwrap();
        let bytes = amf_str.marshall().unwrap();
        assert_eq!(bytes, &[0x00, 0x05, b'h', b'e', b'l', b'l', b'o']);
    }

    // 测试序列化（LBW=4）
    #[test]
    fn try_into_bytes_w4() {
        let amf_str = AmfUtf8::<4>::new_from_str("world").unwrap();
        let bytes = amf_str.marshall().unwrap();
        assert_eq!(
            bytes,
            &[0x00, 0x00, 0x00, 0x05, b'w', b'o', b'r', b'l', b'd']
        );
    }

    // 测试反序列化（LBW=2）
    #[test]
    fn try_from_bytes_w2() {
        let data = [0x00, 0x05, b'h', b'e', b'l', b'l', b'o'];
        let (amf_str, consumed) = AmfUtf8::<2>::unmarshall(&data).unwrap();
        assert_eq!(amf_str.inner, "hello");
        assert_eq!(consumed, 7);
    }

    // 测试反序列化（LBW=4）
    #[test]
    fn try_from_bytes_w4() {
        let data = [0x00, 0x00, 0x00, 0x05, b'w', b'o', b'r', b'l', b'd'];
        let (amf_str, consumed) = AmfUtf8::<4>::unmarshall(&data).unwrap();
        assert_eq!(amf_str.inner, "world");
        assert_eq!(consumed, 9);
    }

    // 测试长度计算
    #[test]
    fn length_calculation() {
        let amf_str = AmfUtf8::<2>::new_from_str("abc").unwrap();
        assert_eq!(amf_str.marshall_length(), 2 + 3); // 2字节长度头 + 3字节内容

        let amf_str = AmfUtf8::<4>::new_from_str("abcde").unwrap();
        assert_eq!(amf_str.marshall_length(), 4 + 5); // 4字节长度头 + 5字节内容
    }

    // 测试TryFrom转换
    #[test]
    fn try_from_slice() {
        let data = [0x00, 0x03, b'f', b'o', b'o'];
        let amf_str: AmfUtf8<2> = data[..].try_into().unwrap();
        assert_eq!(amf_str.inner, "foo");
    }

    // 测试Deref和AsRef
    #[test]
    fn deref_and_as_ref() {
        let amf_str = AmfUtf8::<2>::new_from_str("bar").unwrap();
        assert_eq!(&*amf_str, "bar");
        assert_eq!(amf_str.as_ref(), "bar");
    }

    // 测试Display
    #[test]
    fn display_format() {
        let amf_str = AmfUtf8::<2>::new_from_str("test").unwrap();
        assert_eq!(format!("{}", amf_str), "test");
    }

    /// Helper to compute the hash of a value
    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        t.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    fn clone_preserves_equality() {
        let original = AmfUtf8::<2>::new_from_str("hello").unwrap();
        let cloned = original.clone();
        // After cloning, they should be equal
        assert_eq!(original, cloned);
    }

    #[test]
    fn eq_and_neq_behaviour() {
        let a = AmfUtf8::<4>::new_from_str("rust").unwrap();
        let b_same = AmfUtf8::<4>::new_from_str("rust").unwrap();
        let c_diff = AmfUtf8::<4>::new_from_str("Rust").unwrap();

        // Same content should be equal
        assert_eq!(a, b_same);
        // Different case should not be equal
        assert_ne!(a, c_diff);
    }

    #[test]
    fn equal_values_have_same_hash() {
        let x = AmfUtf8::<2>::new_from_str("hash_me").unwrap();
        let y = AmfUtf8::<2>::new_from_str("hash_me").unwrap();

        let hx = calculate_hash(&x);
        let hy = calculate_hash(&y);
        assert_eq!(hx, hy, "Equal values should produce the same hash");
    }

    #[test]
    fn different_values_have_different_hash() {
        let x = AmfUtf8::<2>::new_from_str("foo").unwrap();
        let y = AmfUtf8::<2>::new_from_str("bar").unwrap();

        let hx = calculate_hash(&x);
        let hy = calculate_hash(&y);
        assert_ne!(hx, hy, "Different values should produce different hashes");
    }

    #[test]
    fn clone_preserves_hash() {
        let original = AmfUtf8::<4>::new_from_str("clone_hash").unwrap();
        let cloned = original.clone();

        let h1 = calculate_hash(&original);
        let h2 = calculate_hash(&cloned);
        assert_eq!(
            h1, h2,
            "Cloned instance should have the same hash as original"
        );
    }
}
