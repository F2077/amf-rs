use crate::errors::AmfError;
use crate::traits::{Length, TryFromBytes, TryIntoBytes};
use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AmfUtf8<const W: usize> {
    inner: String,
}

impl<const W: usize> AmfUtf8<W> {
    pub fn new(inner: &str) -> Result<Self, AmfError> {
        debug_assert!(W == 2 || W == 4);
        let len = inner.len();
        if (W == 2 && len > u16::MAX as usize) || (W == 4 && len > u32::MAX as usize) {
            return Err(AmfError::StringTooLong { max: W, got: len });
        }
        Ok(Self {
            inner: inner.to_string(),
        })
    }
}

impl<const W: usize> TryIntoBytes for AmfUtf8<W> {
    fn try_into_bytes(&self) -> Result<&[u8], AmfError> {
        debug_assert!(W == 2 || W == 4);
        let mut vec = Vec::with_capacity(self.length());
        let length_buf = if W == 2 {
            (self.inner.len() as u16).to_be_bytes()
        } else {
            (self.inner.len() as u32).to_be_bytes()
        };
        vec.extend_from_slice(&length_buf);
        vec.extend_from_slice(self.inner.as_bytes());
        Ok(vec.as_slice())
    }
}

impl<const W: usize> Length for AmfUtf8<W> {
    fn length(&self) -> usize {
        debug_assert!(W == 2 || W == 4);
        W + self.inner.len()
    }
}

impl<const W: usize> TryFromBytes for AmfUtf8<W> {
    fn try_from_bytes(buf: &[u8]) -> Result<(Self, usize), AmfError> {
        debug_assert!(W == 2 || W == 4);
        let length = if W == 2 {
            u16::from_be_bytes(
                buf[0..2]
                    .iter()
                    .try_into()?
                    .map_err(|_| AmfError::Custom("failed to parse length (u16) from buf")),
            )
        } else {
            u32::from_be_bytes(
                buf[0..4]
                    .iter()
                    .try_into()?
                    .map_err(|_| AmfError::Custom("failed to parse length (u32) from buf")),
            )
        };

        let start = W;
        let end = start + length;
        if buf.len() < end {
            return Err(AmfError::BufferTooSmall {
                expected: end,
                got: buf.len(),
            });
        }
        let value = std::str::from_utf8(&buf[start..end]).map_err(|e| AmfError::Io(e))?;
        Ok((
            Self {
                inner: value.to_string(),
            },
            end,
        ))
    }
}

// 实现 rust 惯用语("idiom") 方便用户使用

impl<'a, const W: usize> TryInto<&'a [u8]> for AmfUtf8<W> {
    type Error = AmfError;

    fn try_into(self) -> Result<&'a [u8], Self::Error> {
        self.try_into_bytes().map(|v| &v[..])
    }
}

impl<const W: usize> TryFrom<&[u8]> for AmfUtf8<W> {
    type Error = AmfError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::try_from_bytes(value).map(|(v, _)| v)
    }
}

impl<const W: usize> Display for AmfUtf8<W> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}
impl<const W: usize> AsRef<str> for AmfUtf8<W> {
    fn as_ref(&self) -> &str {
        self.inner.as_ref()
    }
}
impl<const W: usize> Deref for AmfUtf8<W> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        Self::as_ref(self)
    }
}
impl<const W: usize> Borrow<str> for AmfUtf8<W> {
    fn borrow(&self) -> &str {
        Self::as_ref(self)
    }
}

// 类型别名

pub type Utf8 = AmfUtf8<2>;
pub type Utf8Long = AmfUtf8<4>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{Length, TryFromBytes, TryIntoBytes};

    // 测试有效字符串创建（W=2）
    #[test]
    fn new_valid_utf8_w2() {
        let s = "a".repeat(u16::MAX as usize);
        let amf_str = AmfUtf8::<2>::new(&s).unwrap();
        assert_eq!(amf_str.inner, s);
    }

    // 测试过长字符串创建（W=2）
    #[test]
    fn new_too_long_utf8_w2() {
        let s = "a".repeat(u16::MAX as usize + 1);
        assert!(matches!(
            AmfUtf8::<2>::new(&s),
            Err(AmfError::StringTooLong { max: 2, got: _ })
        ));
    }

    // 测试有效字符串创建（W=4）
    #[test]
    fn new_valid_utf8_w4() {
        let s = "a".repeat(1000); // 在u32范围内
        let amf_str = AmfUtf8::<4>::new(&s).unwrap();
        assert_eq!(amf_str.inner, s);
    }

    // 测试序列化（W=2）
    #[test]
    fn try_into_bytes_w2() {
        let amf_str = AmfUtf8::<2>::new("hello").unwrap();
        let bytes = amf_str.try_into_bytes().unwrap();
        assert_eq!(bytes, &[0x00, 0x05, b'h', b'e', b'l', b'l', b'o']);
    }

    // 测试序列化（W=4）
    #[test]
    fn try_into_bytes_w4() {
        let amf_str = AmfUtf8::<4>::new("world").unwrap();
        let bytes = amf_str.try_into_bytes().unwrap();
        assert_eq!(
            bytes,
            &[0x00, 0x00, 0x00, 0x05, b'w', b'o', b'r', b'l', b'd']
        );
    }

    // 测试反序列化（W=2）
    #[test]
    fn try_from_bytes_w2() {
        let data = [0x00, 0x05, b'h', b'e', b'l', b'l', b'o'];
        let (amf_str, consumed) = AmfUtf8::<2>::try_from_bytes(&data).unwrap();
        assert_eq!(amf_str.inner, "hello");
        assert_eq!(consumed, 7);
    }

    // 测试反序列化（W=4）
    #[test]
    fn try_from_bytes_w4() {
        let data = [0x00, 0x00, 0x00, 0x05, b'w', b'o', b'r', b'l', b'd'];
        let (amf_str, consumed) = AmfUtf8::<4>::try_from_bytes(&data).unwrap();
        assert_eq!(amf_str.inner, "world");
        assert_eq!(consumed, 9);
    }

    // 测试长度计算
    #[test]
    fn length_calculation() {
        let amf_str = AmfUtf8::<2>::new("abc").unwrap();
        assert_eq!(amf_str.length(), 2 + 3); // 2字节长度头 + 3字节内容

        let amf_str = AmfUtf8::<4>::new("abcde").unwrap();
        assert_eq!(amf_str.length(), 4 + 5); // 4字节长度头 + 5字节内容
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
        let amf_str = AmfUtf8::<2>::new("bar").unwrap();
        assert_eq!(&*amf_str, "bar");
        assert_eq!(amf_str.as_ref(), "bar");
    }

    // 测试Display
    #[test]
    fn display_format() {
        let amf_str = AmfUtf8::<2>::new("test").unwrap();
        assert_eq!(format!("{}", amf_str), "\"test\"");
    }
}
