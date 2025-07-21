use crate::amf0::type_marker::TypeMarker;
use crate::errors::AmfError;
use crate::traits::{Marshall, MarshallLength, Unmarshall};
use std::fmt::Display;

pub(crate) trait MarkerType: Sized {
    const TYPE_MARKER: TypeMarker;
}

impl<M: MarkerType> Marshall for M {
    fn marshall(&self) -> Result<Vec<u8>, AmfError> {
        let mut buf = [0u8; 1];
        buf[0] = M::TYPE_MARKER as u8; // 单字节情况下，不需考虑字节序问题
        Ok(buf.to_vec())
    }
}

impl<M: MarkerType> MarshallLength for M {
    fn marshall_length(&self) -> usize {
        1
    }
}

impl<M: MarkerType + Default> Unmarshall for M {
    fn unmarshall(buf: &[u8]) -> Result<(Self, usize), AmfError> {
        if buf.len() < 1 {
            return Err(AmfError::BufferTooSmall {
                want: 1,
                got: buf.len(),
            });
        }
        let type_marker = TypeMarker::try_from(buf[0])?;
        if type_marker != M::TYPE_MARKER {
            return Err(AmfError::TypeMarkerValueMismatch {
                want: M::TYPE_MARKER as u8,
                got: buf[0],
            });
        }
        Ok((M::default(), 1))
    }
}

//	The null type is represented by the null type marker. No further information is encoded
//	for this value.
#[derive(Debug, PartialEq, Default)]
pub struct NullType;

impl MarkerType for NullType {
    const TYPE_MARKER: TypeMarker = TypeMarker::Null;
}

impl Display for NullType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "null")
    }
}

//    The undefined type is represented by the undefined type marker. No further information is encoded
//    for this value.
#[derive(Debug, PartialEq, Default)]
pub struct UndefinedType;

impl MarkerType for UndefinedType {
    const TYPE_MARKER: TypeMarker = TypeMarker::Undefined;
}

impl Display for UndefinedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "undefined")
    }
}
