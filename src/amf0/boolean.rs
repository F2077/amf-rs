use crate::amf0::type_marker::TypeMarker;
use crate::errors::AmfError;
use crate::traits::{Marshall, MarshallLength, Unmarshall};
use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

//	An AMF 0 Boolean type is used to encode a primitive ActionScript 1.0 or 2.0 Boolean or
//	an ActionScript 3.0 Boolean. The Object (non-primitive) version of ActionScript 1.0 or
//	2.0 Booleans are not serializable. A Boolean type marker is followed by an unsigned
//	byte; a zero byte value denotes false while a non-zero byte value (typically 1) denotes
//	true.
#[derive(Debug, Clone, PartialEq)]
pub struct BooleanType {
    type_marker: TypeMarker,
    value: bool,
}

impl BooleanType {
    pub fn new(value: bool) -> Self {
        Self {
            type_marker: TypeMarker::Boolean,
            value,
        }
    }
}

impl Marshall for BooleanType {
    fn marshall(&self) -> Result<Vec<u8>, AmfError> {
        debug_assert!(self.type_marker == TypeMarker::Boolean);
        let mut buf = [0u8; 2];
        buf[0] = self.type_marker as u8; // 单字节情况下不用考虑字节序
        buf[1] = self.value as u8;
        Ok(buf.to_vec())
    }
}

impl MarshallLength for BooleanType {
    fn marshall_length(&self) -> usize {
        2 // 1 byte for type marker + 1 byte for value
    }
}

impl Unmarshall for BooleanType {
    fn unmarshall(buf: &[u8]) -> Result<(Self, usize), AmfError> {
        if buf.len() < 2 {
            return Err(AmfError::BufferTooSmall {
                want: 2,
                got: buf.len(),
            });
        }
        let type_marker = TypeMarker::try_from(buf[0])?; // 这里直接用了 buf[0] 是应为单字节情况下不用考虑字节序
        if type_marker != TypeMarker::Boolean {
            return Err(AmfError::TypeMarkerValueMismatch {
                want: TypeMarker::Boolean as u8,
                got: buf[0],
            });
        }
        let value = buf[1] != 0;
        Ok((Self { type_marker, value }, 2))
    }
}

impl TryFrom<&[u8]> for BooleanType {
    type Error = AmfError;

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        Self::unmarshall(buf).map(|(b, _)| b)
    }
}

impl From<bool> for BooleanType {
    fn from(value: bool) -> Self {
        Self::new(value)
    }
}

impl AsRef<bool> for BooleanType {
    fn as_ref(&self) -> &bool {
        &self.value
    }
}

impl Deref for BooleanType {
    type Target = bool;

    fn deref(&self) -> &bool {
        self.as_ref()
    }
}

impl Display for BooleanType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Default for BooleanType {
    fn default() -> Self {
        Self::new(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amf0::type_marker::TypeMarker;
    use crate::errors::AmfError;
    use std::convert::TryFrom;
    use std::fmt::Write as _;
    // for Display tests

    #[test]
    fn boolean_round_trip_true() {
        let orig = BooleanType::new(true);
        let bytes = orig.marshall().expect("marshall should succeed");
        // [marker, value]
        assert_eq!(bytes, vec![TypeMarker::Boolean as u8, 1]);
        // unmarshall
        let (decoded, len) = BooleanType::unmarshall(&bytes).expect("unmarshall should succeed");
        assert_eq!(len, 2);
        assert_eq!(decoded.value, true);
        // TryFrom
        let from_buf = BooleanType::try_from(&bytes[..]).unwrap();
        assert_eq!(from_buf.value, true);
        // From<bool>
        let from_bool: BooleanType = false.into();
        assert_eq!(from_bool.value, false);
        // AsRef, Deref
        assert_eq!(orig.as_ref(), &true);
        assert_eq!(*orig, true);
        // Display
        let mut s = String::new();
        write!(&mut s, "{}", orig).unwrap();
        assert_eq!(s, "true");
    }

    #[test]
    fn boolean_round_trip_false() {
        let orig = BooleanType::new(false);
        let bytes = orig.marshall().unwrap();
        assert_eq!(bytes, vec![TypeMarker::Boolean as u8, 0]);
        let (decoded, _) = BooleanType::unmarshall(&bytes).unwrap();
        assert!(!decoded.value);
    }

    #[test]
    fn boolean_unmarshall_errors() {
        // too short
        let err = BooleanType::unmarshall(&[TypeMarker::Boolean as u8]).unwrap_err();
        match err {
            AmfError::BufferTooSmall { want, got } => {
                assert_eq!(want, 2);
                assert_eq!(got, 1);
            }
            _ => panic!("expected BufferTooSmall"),
        }
        // wrong marker
        let bad = vec![TypeMarker::Number as u8, 1];
        let err2 = BooleanType::unmarshall(&bad).unwrap_err();
        match err2 {
            AmfError::TypeMarkerValueMismatch { want, got } => {
                assert_eq!(want, TypeMarker::Boolean as u8);
                assert_eq!(got, TypeMarker::Number as u8);
            }
            _ => panic!("expected TypeMarkerValueMismatch"),
        }
    }
}
