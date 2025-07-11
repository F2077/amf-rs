use crate::traits::{FromBytes, ToBytes};
use crate::type_marker::TypeMarker;

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
