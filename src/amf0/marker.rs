use crate::amf0::type_marker::TypeMarker;
use crate::errors::AmfError;
use crate::traits::{Marshall, MarshallLength, Unmarshall};
use std::fmt::Display;

pub trait MarkerType: Sized {
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

impl TryFrom<&[u8]> for NullType {
    type Error = AmfError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::unmarshall(value).map(|(o, _)| o)
    }
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

impl TryFrom<&[u8]> for UndefinedType {
    type Error = AmfError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::unmarshall(value).map(|(o, _)| o)
    }
}

impl Display for UndefinedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "undefined")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amf0::type_marker::TypeMarker;

    // NullType 测试
    #[test]
    fn test_null_marshall() {
        let null = NullType;
        let data = null.marshall().unwrap();
        assert_eq!(data, vec![TypeMarker::Null as u8]);
    }

    #[test]
    fn test_null_marshall_length() {
        let null = NullType;
        assert_eq!(null.marshall_length(), 1);
    }

    #[test]
    fn test_null_unmarshall_valid() {
        let data = [TypeMarker::Null as u8];
        let (null, bytes_read) = NullType::unmarshall(&data).unwrap();
        assert_eq!(bytes_read, 1);
        assert_eq!(null, NullType);
    }

    #[test]
    fn test_null_unmarshall_buffer_too_small() {
        let data = [];
        let result = NullType::unmarshall(&data);
        assert!(matches!(
            result,
            Err(AmfError::BufferTooSmall { want: 1, got: 0 })
        ));
    }

    #[test]
    fn test_null_try_from() {
        let data = [TypeMarker::Null as u8];
        let null = NullType::try_from(&data[..]).unwrap();
        assert_eq!(null, NullType);
    }

    #[test]
    fn test_null_display() {
        assert_eq!(format!("{}", NullType), "null");
    }

    // UndefinedType 测试
    #[test]
    fn test_undefined_marshall() {
        let undefined = UndefinedType;
        let data = undefined.marshall().unwrap();
        assert_eq!(data, vec![TypeMarker::Undefined as u8]);
    }

    #[test]
    fn test_undefined_marshall_length() {
        let undefined = UndefinedType;
        assert_eq!(undefined.marshall_length(), 1);
    }

    #[test]
    fn test_undefined_unmarshall_valid() {
        let data = [TypeMarker::Undefined as u8];
        let (undefined, bytes_read) = UndefinedType::unmarshall(&data).unwrap();
        assert_eq!(bytes_read, 1);
        assert_eq!(undefined, UndefinedType);
    }

    #[test]
    fn test_undefined_unmarshall_buffer_too_small() {
        let data = [];
        let result = UndefinedType::unmarshall(&data);
        assert!(matches!(
            result,
            Err(AmfError::BufferTooSmall { want: 1, got: 0 })
        ));
    }

    #[test]
    fn test_undefined_try_from() {
        let data = [TypeMarker::Undefined as u8];
        let undefined = UndefinedType::try_from(&data[..]).unwrap();
        assert_eq!(undefined, UndefinedType);
    }

    #[test]
    fn test_undefined_display() {
        assert_eq!(format!("{}", UndefinedType), "undefined");
    }

    // 泛型实现的额外测试
    #[test]
    fn test_generic_marker_type() {
        // 验证 NullType 的标记
        assert_eq!(NullType::TYPE_MARKER, TypeMarker::Null);

        // 验证 UndefinedType 的标记
        assert_eq!(UndefinedType::TYPE_MARKER, TypeMarker::Undefined);
    }
}
