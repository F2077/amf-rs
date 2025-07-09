use std::borrow::Cow;
use std::io;
use std::io::{BufWriter, Write};

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
    value: Cow<'a, str>, // 智能指针能在性能与易用性之间取得平衡
}

impl<'a> Utf8<'a> {
    pub fn new(value: Cow<'a, str>) -> Self {
        let length = value.len() as u16;
        Utf8 { length, value }
    }

    pub fn new_owned(value: String) -> Self {
        Utf8::new(Cow::Owned(value))
    }

    pub fn new_borrowed(value: &'a str) -> Self {
        Utf8::new(Cow::Borrowed(value))
    }
}

impl<'a> Utf8<'a> {
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(2 + self.length as usize);
        buf.extend_from_slice(self.length.to_be_bytes().as_slice());
        buf.extend_from_slice(self.value.as_bytes());
        buf
    }

    pub fn serialize_to(&self, buf: &mut [u8]) {
        buf[0..2].copy_from_slice(self.length.to_be_bytes().as_slice());
        buf[2..2 + self.length as usize].copy_from_slice(self.value.as_bytes());
    }

    pub fn serialize_size(&self) -> u16 {
        2 + self.length
    }
}

impl<'a> Utf8<'a> {
    pub fn from_bytes_borrowed(buf: &'a [u8]) -> Result<Self, io::Error> {
        let (len, val) = parse(buf)?;
        Ok(Self {
            length: len,
            value: Cow::Borrowed(val),
        })
    }

    pub fn from_bytes_owned(buf: &[u8]) -> Result<Self, io::Error> {
        let (len, val) = parse(buf)?;
        Ok(Self {
            length: len,
            value: Cow::Owned(val.to_string()),
        })
    }
}

impl<'a> Utf8<'a> {
    pub fn deserialize_borrowed(&mut self, buf: &'a [u8]) -> Result<(), io::Error> {
        let (len, val) = parse(buf)?;

        self.length = len;
        self.value = Cow::Borrowed(val);

        Ok(())
    }

    pub fn deserialize_owned(&mut self, buf: &[u8]) -> Result<(), std::io::Error> {
        let (len, val) = parse(buf)?;

        self.length = len;
        self.value = Cow::Owned(val.to_string());

        Ok(())
    }
}

fn parse(buf: &[u8]) -> io::Result<(u16, &str)> {
    if buf.len() < 2 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Not enough data(must have 2 bytes for length)",
        ));
    }
    let length_array = [buf[0], buf[1]];
    let length = u16::from_be_bytes(length_array);

    if buf.len() < 2 + length as usize {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Not enough data(must have 2 + length bytes for value)",
        ));
    }
    let value = std::str::from_utf8(&buf[2..2 + length as usize])
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    Ok((length, value))
}

pub struct Utf8Writer<W: Write> {
    inner: BufWriter<W>,
}

impl<W: Write> Utf8Writer<W> {
    pub fn new(inner: W) -> Utf8Writer<W> {
        Utf8Writer {
            inner: BufWriter::new(inner),
        }
    }

    pub fn write(&mut self, value: &str) -> std::io::Result<()> {
        self.inner.write_all(value.len().to_be_bytes().as_slice())?;
        self.inner.write_all(value.as_bytes())
    }
}
