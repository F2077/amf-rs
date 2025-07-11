use crate::traits::{FromBytes, ToBytes};
use crate::type_marker::TypeMarker;
use crate::utf8::Utf8Short;

// An AMF 0 Number type is used to encode an ActionScript Number.
// The data following a Number type marker is always an 8 byte IEEE-754 double precision floating point value in network byte order (sign bit in low memory).
#[derive(Debug, PartialEq)]
pub struct NumberType {
    type_marker: TypeMarker,
    value: f64,
}

impl ToBytes for NumberType {
    fn to_bytes(&self) -> std::io::Result<Vec<u8>> {
        let mut vec = Vec::with_capacity(1 + 8); // 1 byte for type marker + 8 bytes for value
        vec.push(self.type_marker as u8);
        vec.extend_from_slice(&self.value.to_be_bytes());
        Ok(vec)
    }

    fn bytes_size(&self) -> usize {
        1 + 8
    }

    fn write_bytes_to(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.len() < 9 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Buffer is too small, need at least 9 bytes",
            ));
        }

        buf[0] = self.type_marker as u8;
        buf[1..9].copy_from_slice(&self.value.to_be_bytes());
        Ok(9)
    }
}

impl FromBytes for NumberType {
    fn from_bytes(buf: &[u8]) -> std::io::Result<Self> {
        if buf.len() < 9 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Buffer is too small, need at least 9 bytes",
            ));
        }
        let type_marker = TypeMarker::try_from(buf[0])
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let value = f64::from_be_bytes(buf[1..9].try_into().unwrap());
        Ok(Self { type_marker, value })
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

impl ToBytes for BooleanType {
    fn to_bytes(&self) -> std::io::Result<Vec<u8>> {
        let mut vec = Vec::with_capacity(1 + 1); // 1 byte for type marker + 1 byte for value
        vec.push(self.type_marker as u8);
        vec.push(self.value as u8);
        Ok(vec)
    }

    fn bytes_size(&self) -> usize {
        1 + 1
    }

    fn write_bytes_to(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.len() < 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Buffer is too small, need at least 2 bytes",
            ));
        }
        buf[0] = self.type_marker as u8;
        buf[1] = self.value as u8;
        Ok(2)
    }
}

impl FromBytes for BooleanType {
    fn from_bytes(buf: &[u8]) -> std::io::Result<Self> {
        if buf.len() < 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Buffer is too small, need at least 2 bytes",
            ));
        }
        let type_marker = TypeMarker::try_from(buf[0])
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let value = buf[1] != 0;
        Ok(Self { type_marker, value })
    }
}

//	All strings in AMF are encoded using UTF-8; however, the byte-length header format
//	may vary. The AMF 0 String type uses the standard byte-length header (i.e. U16). For
//	long Strings that require more than 65535 bytes to encode in UTF-8, the AMF 0 Long
//	String type should be used.
pub struct StringType<'a> {
    type_marker: TypeMarker,
    value: Utf8Short<'a>,
}

impl<'a> ToBytes for StringType<'a> {
    fn to_bytes(&self) -> std::io::Result<Vec<u8>> {
        let mut vec = Vec::with_capacity(self.bytes_size());
        vec.push(self.type_marker as u8);
        &self.value.write_bytes_to(&mut vec[1..])?;
        Ok(vec)
    }

    fn bytes_size(&self) -> usize {
        1 + self.value.bytes_size()
    }

    fn write_bytes_to(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        let required_size = 1 + self.value.bytes_size();
        if buf.len() < required_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Buffer is too small, need at least {} bytes", required_size),
            ));
        }
        buf[0] = self.type_marker as u8;
        let n = &self.value.write_bytes_to(&mut buf[1..])?;

        Ok(1 + n)
    }
}
