use std::borrow::Cow;
use std::io;

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

        buffer[0..2].copy_from_slice(self.length.to_be_bytes().as_slice());
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
