use std::borrow::{Borrow, Cow};
use std::fmt::{Display, Formatter};
use std::io;
use std::ops::Deref;

#[derive(Debug, PartialEq)]
pub struct Utf8<'a> {
    length: u16,
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

impl<'a> Utf8<'a> {
    pub fn new(value: Cow<'a, str>) -> Result<Self, io::Error> {
        let length = value.len();
        if length > u16::MAX as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("AMF utf8 length must be less than {}", u16::MAX),
            ));
        }
        Ok(Utf8 {
            length: length as u16,
            value,
        })
    }

    pub fn new_owned(value: String) -> Result<Self, io::Error> {
        Utf8::new(Cow::Owned(value))
    }

    pub fn new_borrowed(value: &'a str) -> Result<Self, io::Error> {
        Utf8::new(Cow::Borrowed(value))
    }
}

impl<'a> Utf8<'a> {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(2 + self.length as usize);
        buf.extend_from_slice(self.length.to_be_bytes().as_slice());
        buf.extend_from_slice(self.value.as_bytes());
        buf
    }

    pub fn bytes_size(&self) -> u16 {
        2 + self.length
    }

    pub fn write_to(&self, buffer: &mut [u8]) -> Result<usize, io::Error> {
        let required_size = self.bytes_size() as usize;
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

        buffer[0..2].copy_from_slice(self.length.to_be_bytes().as_slice()); // copy_from_slice 在底层通常会由编译器优化为高效的 memcpy 操作
        buffer[2..2 + self.length as usize].copy_from_slice(self.value.as_bytes());

        Ok(required_size)
    }
}

impl<'a> Utf8<'a> {
    // 实现了零拷贝反序列化，当输入 &[u8] 的生命周期足够长时，可以直接借用其数据，避免了不必要的内存分配
    pub fn from_bytes_borrowed(buf: &'a [u8]) -> Result<Self, io::Error> {
        let (len, val) = Self::parse(buf)?;
        Ok(Self {
            length: len,
            value: Cow::Borrowed(val),
        })
    }

    // 提供了灵活性，在需要时可以创建一份拥有所有权的数据副本
    pub fn from_bytes_owned(buf: &[u8]) -> Result<Self, io::Error> {
        let (len, val) = Self::parse(buf)?;
        Ok(Self {
            length: len,
            value: Cow::Owned(val.to_string()),
        })
    }

    fn parse(buf: &[u8]) -> io::Result<(u16, &str)> {
        if buf.len() < 2 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Insufficient data for length header",
            ));
        }

        let length = u16::from_be_bytes([buf[0], buf[1]]);
        let data_start = 2;
        let data_end = data_start + length as usize;

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
impl<'a> Deref for Utf8<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

// 提供从 &Utf8 到 &str 的显式转换
impl<'a> AsRef<str> for Utf8<'a> {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

// 允许在需要 &str 的地方（如 HashMap 的 key）使用 &Utf8
impl<'a> Borrow<str> for Utf8<'a> {
    fn borrow(&self) -> &str {
        &self.value
    }
}

// 允许直接打印 Utf8 实例
impl<'a> Display for Utf8<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

// 提供符合惯例的、可能失败的转换方式 (from a borrowed string)
impl<'a> TryFrom<&'a str> for Utf8<'a> {
    type Error = io::Error;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Utf8::new_borrowed(value)
    }
}

// 提供符合惯例的、可能失败的转换方式 (from an owned string)
impl<'a> TryFrom<String> for Utf8<'a> {
    type Error = io::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Utf8::new_owned(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn test_new_borrowed_success() {
        let s = "hello";
        let utf8 = Utf8::new_borrowed(s).unwrap();
        assert_eq!(utf8.length, 5);
        assert_eq!(utf8.value.as_ref(), "hello");
        assert!(matches!(utf8.value, Cow::Borrowed(_)));
    }

    #[test]
    fn test_new_owned_success() {
        let s = "world".to_string();
        let utf8 = Utf8::new_owned(s).unwrap();
        assert_eq!(utf8.length, 5);
        assert_eq!(utf8.value.as_ref(), "world");
        assert!(matches!(utf8.value, Cow::Owned(_)));
    }

    #[test]
    fn test_string_too_long() {
        let long_string = "a".repeat(u16::MAX as usize + 1);
        let result = Utf8::new_owned(long_string);
        assert!(result.is_err());
    }

    #[test]
    fn test_roundtrip_borrowed() {
        let s = "你好, world!";
        let utf8 = Utf8::new_borrowed(s).unwrap();

        let bytes = utf8.to_bytes();
        let parsed_utf8 = Utf8::from_bytes_borrowed(&bytes).unwrap();

        assert_eq!(utf8, parsed_utf8);
        assert_eq!(parsed_utf8.as_ref(), s);
        assert!(matches!(parsed_utf8.value, Cow::Borrowed(_)));
    }

    #[test]
    fn test_roundtrip_owned() {
        let s = "你好, world!".to_string();
        let utf8 = Utf8::new_owned(s.clone()).unwrap();

        let bytes = utf8.to_bytes();
        let parsed_utf8 = Utf8::from_bytes_owned(&bytes).unwrap();

        assert_eq!(utf8, parsed_utf8);
        assert_eq!(*parsed_utf8, s);
        assert!(matches!(parsed_utf8.value, Cow::Owned(_)));
    }

    #[test]
    fn test_write_to_and_parse() {
        let s = "test write_to";
        let utf8 = Utf8::new_borrowed(s).unwrap();
        let mut buffer = vec![0; utf8.bytes_size() as usize];

        let bytes_written = utf8.write_to(&mut buffer).unwrap();
        assert_eq!(bytes_written, buffer.len());

        let parsed = Utf8::from_bytes_borrowed(&buffer).unwrap();
        assert_eq!(parsed.as_ref(), s);
    }

    #[test]
    fn test_write_to_small_buffer() {
        let s = "short";
        let utf8 = Utf8::new_borrowed(s).unwrap();
        let mut buffer = vec![0; 4]; // Too small

        let result = utf8.write_to(&mut buffer);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn test_parse_insufficient_header() {
        let bytes = vec![0x00]; // Only 1 byte
        let result = Utf8::from_bytes_borrowed(&bytes);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn test_parse_insufficient_data() {
        let bytes = vec![0x00, 0x0A, b'h', b'e', b'l', b'l', b'o']; // Declares length 10, but provides 5
        let result = Utf8::from_bytes_borrowed(&bytes);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn test_parse_invalid_utf8() {
        // A byte slice with length prefix followed by invalid UTF-8 sequence
        let bytes = vec![0x00, 0x04, 0xff, 0xff, 0xff, 0xff];
        let result = Utf8::from_bytes_borrowed(&bytes);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn test_empty_string_roundtrip() {
        let s = "";
        let utf8 = Utf8::new_borrowed(s).unwrap();
        assert_eq!(utf8.length, 0);
        let bytes = utf8.to_bytes();
        assert_eq!(bytes, vec![0x00, 0x00]);
        let parsed = Utf8::from_bytes_borrowed(&bytes).unwrap();
        assert_eq!(parsed.as_ref(), "");
    }

    #[test]
    fn test_try_from_trait() {
        let s = "hello from trait";
        let utf8 = Utf8::try_from(s).unwrap();
        assert_eq!(utf8.as_ref(), s);

        let s_owned = "owned trait".to_string();
        let utf8_owned = Utf8::try_from(s_owned).unwrap();
        assert_eq!(utf8_owned.as_ref(), "owned trait");
    }

    #[test]
    fn test_deref_and_as_ref() {
        let s = "check deref";
        let utf8 = Utf8::new_borrowed(s).unwrap();
        // Deref in action
        assert!(utf8.starts_with("check"));
        // AsRef in action
        fn takes_str_ref(_s: &str) {}
        takes_str_ref(utf8.as_ref());
    }
}
