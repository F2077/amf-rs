use crate::amf0::type_marker::TypeMarker;
use crate::errors::AmfError;
use crate::traits::{Marshall, MarshallLength, Unmarshall};
use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

// An AMF 0 Number type is used to encode an ActionScript Number.
// The data following a Number type marker is always an 8 byte IEEE-754 double precision floating point value in network byte order (sign bit in low memory).
#[derive(Debug, Clone, PartialEq)]
pub struct NumberType {
    type_marker: TypeMarker,
    value: f64,
}

impl NumberType {
    pub fn new(value: f64) -> Self {
        Self {
            type_marker: TypeMarker::Number,
            value,
        }
    }
}

impl Marshall for NumberType {
    fn marshall(&self) -> Result<Vec<u8>, AmfError> {
        debug_assert!(self.type_marker == TypeMarker::Number);
        let mut buf = [0u8; 9];
        buf[0] = self.type_marker as u8;
        buf[1..9].copy_from_slice(&self.value.to_be_bytes());
        Ok(buf.to_vec())
    }
}

impl MarshallLength for NumberType {
    fn marshall_length(&self) -> usize {
        1 + 8 // 1 byte for type marker + 8 bytes for value
    }
}

impl Unmarshall for NumberType {
    fn unmarshall(buf: &[u8]) -> Result<(Self, usize), AmfError> {
        if buf.len() < 9 {
            return Err(AmfError::BufferTooSmall {
                want: 9,
                got: buf.len(),
            });
        }
        let type_marker = TypeMarker::try_from(buf[0])?;
        if type_marker != TypeMarker::Number {
            return Err(AmfError::TypeMarkerValueMismatch {
                want: TypeMarker::Number as u8,
                got: buf[0],
            });
        }
        let value = f64::from_be_bytes(buf[1..9].try_into().unwrap()); // 前边已经校验了 buf 的长度，这里直接用 .unwrap() 是安全的
        Ok((Self { type_marker, value }, 9))
    }
}

impl TryFrom<&[u8]> for NumberType {
    type Error = AmfError;

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        Self::unmarshall(buf).map(|(n, _)| n)
    }
}

impl From<f64> for NumberType {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

impl AsRef<f64> for NumberType {
    fn as_ref(&self) -> &f64 {
        &self.value
    }
}

impl Deref for NumberType {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Borrow<f64> for NumberType {
    fn borrow(&self) -> &f64 {
        self.as_ref()
    }
}

impl Display for NumberType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
