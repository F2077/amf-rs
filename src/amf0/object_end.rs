use crate::amf0::type_marker::TypeMarker;
use crate::amf0::utf8::Utf8;
use crate::errors::AmfError;
use crate::traits::{Marshall, MarshallLength, Unmarshall};
use std::fmt::{Display, Formatter};

//	The object-end-marker is used in a special type that signals the end of a set of object
//	properties in an anonymous object or typed object or associative array. It is not expected
//	outside of these types. This marker is always preceded by an empty UTF-8 string and
//	together forms the object end type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
        let mut vec = Vec::with_capacity(self.marshall_length());
        vec.extend_from_slice(&self.empty.marshall()?);
        vec.push(self.type_marker as u8);
        Ok(vec)
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

// 实现 rust 惯用语("idiom") 方便用户使用

impl TryFrom<&[u8]> for ObjectEndType {
    type Error = AmfError;

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        Self::unmarshall(buf).map(|(o, _)| o)
    }
}

impl TryFrom<Vec<u8>> for ObjectEndType {
    type Error = AmfError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_slice())
    }
}

impl TryFrom<ObjectEndType> for Vec<u8> {
    type Error = AmfError;

    fn try_from(value: ObjectEndType) -> Result<Self, Self::Error> {
        value.marshall()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amf0::type_marker::TypeMarker;

    #[test]
    fn test_new() {
        let obj_end = ObjectEndType::new();
        assert_eq!(obj_end.empty, Utf8::default());
        assert_eq!(obj_end.type_marker, TypeMarker::ObjectEnd);
    }

    #[test]
    fn test_default() {
        let obj_end1 = ObjectEndType::default();
        let obj_end2 = ObjectEndType::new();
        assert_eq!(obj_end1, obj_end2);
    }

    #[test]
    fn test_marshall() {
        let obj_end = ObjectEndType::new();
        let data = obj_end.marshall().unwrap();
        assert_eq!(data, vec![0x00, 0x00, 0x09]); // 0x09 = ObjectEnd marker
    }

    #[test]
    fn test_marshall_length() {
        let obj_end = ObjectEndType::new();
        assert_eq!(obj_end.marshall_length(), 3);
    }

    #[test]
    fn test_unmarshall_valid() {
        let data = [0x00, 0x00, 0x09];
        let (obj_end, bytes_read) = ObjectEndType::unmarshall(&data).unwrap();
        assert_eq!(bytes_read, 3);
        assert_eq!(obj_end.empty, Utf8::default());
        assert_eq!(obj_end.type_marker, TypeMarker::ObjectEnd);
    }

    #[test]
    fn test_unmarshall_buffer_too_small() {
        let data = [0x00, 0x00]; // 缺少类型标记
        let result = ObjectEndType::unmarshall(&data);
        assert!(matches!(
            result,
            Err(AmfError::BufferTooSmall { want: 3, got: 2 })
        ));
    }

    #[test]
    fn test_unmarshall_invalid_marker() {
        let data = [0x00, 0x00, 0x01]; // 0x01 是无效的结束标记
        let result = ObjectEndType::unmarshall(&data);
        assert!(matches!(
            result,
            Err(AmfError::TypeMarkerValueMismatch {
                want: 0x09,
                got: 0x01
            })
        ));
    }

    #[test]
    fn test_try_from_slice() {
        let data = [0x00, 0x00, 0x09];
        let obj_end = ObjectEndType::try_from(&data[..]).unwrap();
        assert_eq!(obj_end, ObjectEndType::new());
    }

    #[test]
    fn test_display() {
        let obj_end = ObjectEndType::new();
        assert_eq!(format!("{}", obj_end), "");
    }

    #[test]
    fn test_partial_eq() {
        let obj_end1 = ObjectEndType::new();
        let obj_end2 = ObjectEndType::default();
        assert_eq!(obj_end1, obj_end2);
    }

    #[test]
    fn test_clone_eq() {
        let original = ObjectEndType::new();
        let cloned = original.clone();
        assert_eq!(cloned, original);
        assert!(!std::ptr::eq(&original, &cloned));
    }
}
