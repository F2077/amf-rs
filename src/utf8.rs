use crate::traits::{FromBytes, FromBytesRef, ToBytes};
use std::borrow::{Borrow, Cow};
use std::fmt::{Display, Formatter};
use std::io;
use std::ops::Deref;

pub trait Length: Copy + Sized {
    const WIDTH: usize;
    const MAX: usize;
    fn write_be_bytes(self, buf: &mut [u8]) -> io::Result<()>;
    fn read_be_bytes(buf: &[u8]) -> io::Result<Self>;
}

macro_rules! impl_length_for {
    ($type:ty) => {
        impl Length for $type {
            const WIDTH: usize = std::mem::size_of::<$type>();
            const MAX: usize = <$type>::MAX as usize;

            fn write_be_bytes(self, buf: &mut [u8]) -> io::Result<()> {
                if buf.len() < Self::WIDTH {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!(
                            "buffer too small: need {} bytes, got {}",
                            Self::WIDTH,
                            buf.len()
                        ),
                    ));
                }
                buf[..Self::WIDTH].copy_from_slice(&self.to_be_bytes());
                Ok(())
            }

            fn read_be_bytes(buf: &[u8]) -> io::Result<Self> {
                if buf.len() < Self::WIDTH {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        format!("not enough bytes for {}", stringify!($type)),
                    ));
                }
                Ok(Self::from_be_bytes(buf[0..Self::WIDTH].try_into().unwrap()))
            }
        }
    };
}

impl_length_for!(u16);
impl_length_for!(u32);

pub type Utf8<'a> = AmfUtf8<'a, u16>;
pub type Utf8Long<'a> = AmfUtf8<'a, u32>;

pub const EMPTY_UTF8: Utf8<'static> = AmfUtf8 {
    length: 0u16,
    value: Cow::Borrowed(""),
};
pub const EMPTY_UTF8_LONG: Utf8Long<'static> = AmfUtf8 {
    length: 0u32,
    value: Cow::Borrowed(""),
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct AmfUtf8<'a, L: Length> {
    length: L,
    /// UTF-8 string value storage field
    ///
    /// ## Storage Characteristics
    /// - When using `Cow::Borrowed`:
    ///   ```
    ///   +------------------+
    ///   | ptr: *const u8   | → Points to external data
    ///   | len: usize       |
    ///   +------------------+
    ///   ```
    /// - When using `Cow::Owned`:
    ///   ```
    ///   +------------------+
    ///   | ptr: *mut u8     | → Heap-allocated memory
    ///   | len: usize       |
    ///   | cap: usize       |
    ///   +------------------+
    ///   ```
    ///
    /// ## Lifetime Constraints
    /// ```mermaid
    /// graph LR
    ///     A[Source Data] --> B[Struct Instance]
    ///     B --> C[Serialized Result]
    ///     style A fill:#f9f,stroke:#333
    ///     style B fill:#9f9,stroke:#333
    /// ```
    value: Cow<'a, str>, // 智能指针能在性能与灵活性之间取得平衡
}

impl<'a, L> AmfUtf8<'a, L>
where
    L: Length + TryFrom<usize>,
{
    pub fn new(value: Cow<'a, str>) -> Result<Self, io::Error> {
        let len = value.len();
        if len > L::MAX {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("AMF utf8 length {} exceeds max({})", len, L::MAX),
            ));
        }
        let length = L::try_from(len).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Length conversion failed unexpectedly",
            )
        })?;
        Ok(Self { length, value })
    }

    pub fn new_owned(value: String) -> Result<Self, io::Error> {
        Self::new(Cow::Owned(value))
    }

    pub fn new_borrowed(value: &'a str) -> Result<Self, io::Error> {
        Self::new(Cow::Borrowed(value))
    }
}

impl<L> AmfUtf8<'static, L>
where
    L: Length + TryFrom<usize>,
{
    pub fn new_static(value: &'static str) -> Result<Self, io::Error> {
        let len = value.len();
        if len > L::MAX {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("AMF utf8 length {} exceeds max({})", len, L::MAX),
            ));
        }
        let length = L::try_from(len).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Length conversion failed unexpectedly",
            )
        })?;
        Ok(Self {
            length,
            value: Cow::Borrowed(value),
        })
    }

    pub fn new_empty() -> Result<Self, io::Error> {
        Self::new_static("")
    }
}

impl<'a, L> ToBytes for AmfUtf8<'a, L>
where
    L: Length + TryInto<usize>,
{
    fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(self.bytes_size());

        buf.resize(L::WIDTH, 0);
        self.length.write_be_bytes(&mut buf)?;
        buf.extend_from_slice(self.value.as_bytes());
        Ok(buf)
    }

    fn bytes_size(&self) -> usize {
        L::WIDTH + self.value.len()
    }

    fn write_bytes_to(&self, buffer: &mut [u8]) -> Result<usize, io::Error> {
        let required_size = self.bytes_size();
        let buffer_len = buffer.len();
        if buffer_len < required_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Buffer is too small, required size is {} bytes, buffer size is {} bytes",
                    required_size, buffer_len
                ),
            ));
        }

        self.length.write_be_bytes(&mut buffer[0..L::WIDTH])?;
        let len = self.length.try_into().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Length conversion failed unexpectedly",
            )
        })?;
        buffer[L::WIDTH..L::WIDTH + len].copy_from_slice(self.value.as_bytes()); // copy_from_slice 在底层通常会由编译器优化为高效的 memcpy 操作

        Ok(required_size)
    }
}

impl<'a, L> FromBytes for AmfUtf8<'a, L>
where
    L: Length + TryInto<usize>,
{
    // 在需要时可以创建一份拥有所有权的数据副本(提供了灵活性)
    fn from_bytes(buf: &[u8]) -> Result<(Self, usize), io::Error> {
        let (length, val) = Self::parse(buf)?;
        let len = L::WIDTH
            + length.try_into().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Length conversion failed unexpectedly",
                )
            })?;
        Ok((
            Self {
                length,
                value: Cow::Owned(val.to_string()),
            },
            len,
        ))
    }
}

impl<'a, L> FromBytesRef<'a> for AmfUtf8<'a, L>
where
    L: Length + TryInto<usize>,
{
    // 零拷贝反序列化，当输入 &[u8] 的生命周期足够长时，可以直接借用其数据，避免了不必要的内存分配(提供了性能)
    fn from_bytes_ref(buf: &'a [u8]) -> Result<(Self, usize), io::Error> {
        let (length, val) = Self::parse(buf)?;
        let len = L::WIDTH
            + length.try_into().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Length conversion failed unexpectedly",
                )
            })?;
        Ok((
            Self {
                length,
                value: Cow::Borrowed(val),
            },
            len,
        ))
    }
}

impl<'a, L> AmfUtf8<'a, L>
where
    L: Length + TryInto<usize>,
{
    fn parse(buf: &[u8]) -> io::Result<(L, &str)> {
        let length = L::read_be_bytes(buf)?;
        let data_start = L::WIDTH;
        let len = length.try_into().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Length conversion failed unexpectedly",
            )
        })?;
        let data_end = data_start + len;

        if buf.len() < data_end {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!(
                    "Insufficient data for string (expected {} bytes, got {})",
                    data_end,
                    buf.len()
                ),
            ));
        }

        let value = std::str::from_utf8(&buf[data_start..data_end])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok((length, value))
    }
}

// 让 Utf8 可以像 &str 一样被使用（例如：my_utf8.len(), my_utf8.starts_with("...")）
impl<'a, L: Length> Deref for AmfUtf8<'a, L> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

// 提供从 &Utf8 到 &str 的显式转换
impl<'a, L: Length> AsRef<str> for AmfUtf8<'a, L> {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

// 允许在需要 &str 的地方（如 HashMap 的 key）使用 &Utf8
impl<'a, L: Length> Borrow<str> for AmfUtf8<'a, L> {
    fn borrow(&self) -> &str {
        &self.value
    }
}

// 允许直接打印 Utf8 实例
impl<'a, L: Length> Display for AmfUtf8<'a, L> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

// 提供符合惯例的、可能失败的转换方式 (from a borrowed string)
impl<'a, L: Length + TryFrom<usize>> TryFrom<&'a str> for AmfUtf8<'a, L> {
    type Error = io::Error;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        AmfUtf8::new_borrowed(value)
    }
}

// 提供符合惯例的、可能失败的转换方式 (from an owned string)
impl<'a, L: Length + TryFrom<usize>> TryFrom<String> for AmfUtf8<'a, L> {
    type Error = io::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        AmfUtf8::new_owned(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn test_new_borrowed_success() {
        let s = "hello";
        let utf8: AmfUtf8<u16> = AmfUtf8::new_borrowed(s).unwrap();
        assert_eq!(utf8.length, 5);
        assert_eq!(utf8.value.as_ref(), "hello");
        assert!(matches!(utf8.value, Cow::Borrowed(_)));
    }

    #[test]
    fn test_new_owned_success() {
        let s = "world".to_string();
        let utf8: AmfUtf8<u16> = AmfUtf8::new_owned(s).unwrap();
        assert_eq!(utf8.length, 5);
        assert_eq!(utf8.value.as_ref(), "world");
        assert!(matches!(utf8.value, Cow::Owned(_)));
    }

    #[test]
    fn test_string_too_long() {
        let long_string = "a".repeat(u16::MAX as usize + 1);
        let result: Result<AmfUtf8<u16>, io::Error> = AmfUtf8::new_owned(long_string);
        assert!(result.is_err());
    }

    #[test]
    fn test_roundtrip_borrowed() {
        let s = "你好, world!";
        let utf8 = AmfUtf8::new_borrowed(s).unwrap();

        let bytes = utf8.to_bytes().unwrap();
        let (parsed_utf8, consumed) = AmfUtf8::<u16>::from_bytes_ref(&bytes).unwrap();

        assert_eq!(utf8, parsed_utf8);
        assert_eq!(parsed_utf8.as_ref(), s);
        assert_eq!(consumed, bytes.len()); // 验证消耗的字节数
        assert!(matches!(parsed_utf8.value, Cow::Borrowed(_)));
    }

    #[test]
    fn test_roundtrip_owned() {
        let s = "你好, world!".to_string();
        let utf8 = AmfUtf8::new_owned(s.clone()).unwrap();

        let bytes = utf8.to_bytes().unwrap();
        let (parsed_utf8, consumed) = AmfUtf8::<u16>::from_bytes(&bytes).unwrap();

        assert_eq!(utf8, parsed_utf8);
        assert_eq!(*parsed_utf8, s);
        assert_eq!(consumed, bytes.len()); // 验证消耗的字节数
        assert!(matches!(parsed_utf8.value, Cow::Owned(_)));
    }

    #[test]
    fn test_write_to_and_parse() {
        let s = "test write_to";
        let utf8: AmfUtf8<u16> = AmfUtf8::new_borrowed(s).unwrap();
        let mut buffer = vec![0; utf8.bytes_size()];

        let bytes_written = utf8.write_bytes_to(&mut buffer).unwrap();
        assert_eq!(bytes_written, buffer.len());

        let (parsed, _) = AmfUtf8::<u16>::from_bytes_ref(&buffer).unwrap();
        assert_eq!(parsed.as_ref(), s);
    }

    #[test]
    fn test_write_to_small_buffer() {
        let s = "short";
        let utf8: AmfUtf8<u16> = AmfUtf8::new_borrowed(s).unwrap();
        let mut buffer = vec![0; 4]; // Too small

        let result = utf8.write_bytes_to(&mut buffer);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn test_parse_insufficient_header() {
        let bytes = vec![0x00]; // Only 1 byte
        let result: Result<(AmfUtf8<u16>, usize), io::Error> = AmfUtf8::from_bytes_ref(&bytes);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn test_parse_insufficient_data() {
        let bytes = vec![0x00, 0x0A, b'h', b'e', b'l', b'l', b'o']; // Declares length 10, but provides 5
        let result: Result<(AmfUtf8<u16>, usize), io::Error> = AmfUtf8::from_bytes_ref(&bytes);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn test_parse_invalid_utf8() {
        // A byte slice with length prefix followed by invalid UTF-8 sequence
        let bytes = vec![0x00, 0x04, 0xff, 0xff, 0xff, 0xff];
        let result: Result<(AmfUtf8<u16>, usize), io::Error> = AmfUtf8::from_bytes_ref(&bytes);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn test_empty_string_roundtrip() {
        let s = "";
        let utf8: AmfUtf8<u16> = AmfUtf8::new_borrowed(s).unwrap();
        assert_eq!(utf8.length, 0);
        let bytes = utf8.to_bytes().unwrap();
        assert_eq!(bytes, vec![0x00, 0x00]);
        let (parsed, consumed) = AmfUtf8::<u16>::from_bytes_ref(&bytes).unwrap();
        assert_eq!(parsed.as_ref(), "");
        assert_eq!(consumed, 2); // 验证消耗的字节数
    }

    #[test]
    fn test_try_from_trait() {
        let s = "hello from trait";
        let utf8: AmfUtf8<u16> = AmfUtf8::try_from(s).unwrap();
        assert_eq!(utf8.as_ref(), s);

        let s_owned = "owned trait".to_string();
        let utf8_owned: AmfUtf8<u16> = AmfUtf8::try_from(s_owned).unwrap();
        assert_eq!(utf8_owned.as_ref(), "owned trait");
    }

    #[test]
    fn test_deref_and_as_ref() {
        let s = "check deref";
        let utf8: AmfUtf8<u16> = AmfUtf8::new_borrowed(s).unwrap();
        // Deref in action
        assert!(utf8.starts_with("check"));
        // AsRef in action
        fn takes_str_ref(_s: &str) {}
        takes_str_ref(utf8.as_ref());
    }

    #[test]
    fn test_u32_max_length_success() {
        // 创建长度刚好为 u32::MAX 的字符串（仅测试长度值，不实际分配内存）
        let length = u32::MAX;
        let value = "a".repeat(length as usize);
        let utf8: AmfUtf8<u32> = AmfUtf8::new_owned(value.clone()).unwrap();

        assert_eq!(utf8.length, length);
        assert_eq!(utf8.value.len(), length as usize);
    }

    #[test]
    fn test_u32_roundtrip_large_string() {
        let s = "a".repeat(65_536);
        let utf8 = AmfUtf8::<u32>::new_owned(s.clone()).unwrap();

        let bytes = utf8.to_bytes().unwrap();
        let (parsed_utf8, consumed) = AmfUtf8::<u32>::from_bytes(&bytes).unwrap();

        assert_eq!(*parsed_utf8, s);
        assert_eq!(consumed, bytes.len()); // 验证消耗的字节数
    }

    #[test]
    fn test_u32_insufficient_data() {
        // 声明长度 100_000 (0x000186A0)
        let mut bytes = vec![0x00, 0x01, 0x86, 0xA0];
        // 只添加少量数据
        bytes.extend_from_slice(b"insufficient");

        let result: Result<(AmfUtf8<u32>, usize), io::Error> = AmfUtf8::from_bytes_ref(&bytes);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn test_u32_to_bytes() {
        let s = "u32 length test";
        let utf8 = AmfUtf8::<u32>::new_borrowed(s).unwrap();
        let bytes = utf8.to_bytes().unwrap();

        // 验证长度前缀 (4字节大端)
        assert_eq!(bytes[0..4], [0x00, 0x00, 0x00, 0x0F]);
        // 验证字符串数据
        assert_eq!(&bytes[4..], s.as_bytes());
    }

    #[test]
    fn test_u32_write_to_buffer() {
        let s = "buffer write test with u32";
        let utf8 = AmfUtf8::<u32>::new_borrowed(s).unwrap();
        let mut buffer = vec![0; utf8.bytes_size()];

        utf8.write_bytes_to(&mut buffer).unwrap();

        // 解析长度前缀
        let length = u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        assert_eq!(length, s.len() as u32);
        // 验证字符串内容
        assert_eq!(&buffer[4..], s.as_bytes());
    }

    #[test]
    fn test_consumed_length() {
        let s = "hello world";
        let extra_data = [1, 2, 3, 4];

        let utf8 = AmfUtf8::<u16>::new_borrowed(s).unwrap();
        let mut data = utf8.to_bytes().unwrap();
        data.extend_from_slice(&extra_data);

        let (parsed, consumed) = AmfUtf8::<u16>::from_bytes(&data).unwrap();

        assert_eq!(*parsed, s.to_string());
        assert_eq!(consumed, data.len() - extra_data.len()); // 应只消耗字符串部分
    }

    #[test]
    fn test_consumed_length_ref() {
        let s = "hello world";
        let extra_data = [1, 2, 3, 4];

        let utf8 = AmfUtf8::<u16>::new_borrowed(s).unwrap();
        let mut data = utf8.to_bytes().unwrap();
        data.extend_from_slice(&extra_data);

        // 使用from_bytes_ref
        let (parsed, consumed) = AmfUtf8::<u16>::from_bytes_ref(&data).unwrap();

        assert_eq!(*parsed, s.to_string());
        assert_eq!(consumed, data.len() - extra_data.len()); // 应只消耗字符串部分
        assert!(matches!(parsed.value, Cow::Borrowed(_))); // 验证借用
    }
}
