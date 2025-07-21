use crate::amf0::type_marker::TypeMarker;
use crate::amf0::utf8::Utf8;
use crate::errors::AmfError;
use crate::traits::{Marshall, MarshallLength, Unmarshall};
use std::fmt::{Display, Formatter};

//	The object-end-marker is used in a special type that signals the end of a set of object
//	properties in an anonymous object or typed object or associative array. It is not expected
//	outside of these types. This marker is always preceded by an empty UTF-8 string and
//	together forms the object end type.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectEndType {
    empty: Utf8,
    type_marker: TypeMarker,
}

impl ObjectEndType {
    pub fn new() -> Self {
        Self {
            empty: Utf8::default(),
            type_marker: TypeMarker::ObjectEnd,
        }
    }
}

impl Marshall for ObjectEndType {
    fn marshall(&self) -> Result<Vec<u8>, AmfError> {
        debug_assert!(self.type_marker == TypeMarker::ObjectEnd);
        let mut buf = [0u8; 3];
        buf.copy_from_slice(&self.empty.marshall()?);
        buf[2] = self.type_marker as u8;
        Ok(buf.to_vec())
    }
}

impl MarshallLength for ObjectEndType {
    fn marshall_length(&self) -> usize {
        self.empty.marshall_length() + 1
    }
}

impl Unmarshall for ObjectEndType {
    fn unmarshall(buf: &[u8]) -> Result<(Self, usize), AmfError> {
        if buf.len() < 3 {
            return Err(AmfError::BufferTooSmall {
                want: 3,
                got: buf.len(),
            });
        }
        let (empty, _) = Utf8::unmarshall(&buf[0..2])?;
        let type_marker = TypeMarker::try_from(buf[2])?;
        if type_marker != TypeMarker::ObjectEnd {
            return Err(AmfError::TypeMarkerValueMismatch {
                want: TypeMarker::ObjectEnd as u8,
                got: buf[2],
            });
        }
        Ok((Self { empty, type_marker }, 3))
    }
}

impl TryFrom<&[u8]> for ObjectEndType {
    type Error = AmfError;

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        Self::unmarshall(buf).map(|(o, _)| o)
    }
}

impl Display for ObjectEndType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.empty)
    }
}

impl Default for ObjectEndType {
    fn default() -> Self {
        Self::new()
    }
}
