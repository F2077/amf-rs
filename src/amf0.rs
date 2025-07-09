use std::io::{BufWriter, Write};

pub struct Utf8 {
    length: u16,
    value: str,
}

impl Utf8 {
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

    pub fn write_to(&self, writer: &mut dyn Write) -> Result<(), std::io::Error> {
        writer.write_all(self.serialize().as_slice())
    }
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
