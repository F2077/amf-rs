use crate::amf0::type_marker::TypeMarker;
use crate::errors::AmfError;
use crate::traits::{Marshall, MarshallLength, Unmarshall};
use std::fmt::{Display, Formatter};
use std::ops::{Add, Deref};

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

impl Display for NumberType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Default for NumberType {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl Add for NumberType {
    type Output = NumberType;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.value + rhs.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amf0::type_marker::TypeMarker;
    use std::f64::{EPSILON, INFINITY, NAN, NEG_INFINITY};

    #[test]
    fn test_new() {
        let num = NumberType::new(3.14);
        assert_eq!(num.type_marker, TypeMarker::Number);
        assert!((num.value - 3.14).abs() < EPSILON);
    }

    #[test]
    fn test_default() {
        let num = NumberType::default();
        assert_eq!(num.type_marker, TypeMarker::Number);
        assert!((num.value - 0.0).abs() < EPSILON);
    }

    #[test]
    fn test_from_f64() {
        let num: NumberType = 3.14.into();
        assert_eq!(num.type_marker, TypeMarker::Number);
        assert!((num.value - 3.14).abs() < EPSILON);
    }

    #[test]
    fn test_clone_eq() {
        let original = NumberType::new(2.718);
        let cloned = original.clone();
        // Ensure clone produces an equal value
        assert_eq!(cloned, original);
        // Ensure they are distinct instances
        assert!(!std::ptr::eq(&original, &cloned));
    }

    #[test]
    fn test_partial_eq() {
        let a = NumberType::new(1.0);
        let b = NumberType::new(1.0);
        let c = NumberType::new(2.0);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_marshall() {
        let num = NumberType::new(3.14);
        let data = num.marshall().unwrap();

        let expected_marker = TypeMarker::Number as u8;
        let expected_value = 3.14f64.to_be_bytes();

        assert_eq!(data[0], expected_marker);
        assert_eq!(&data[1..9], expected_value);
    }

    #[test]
    fn test_marshall_special_values() {
        // 测试特殊浮点值
        let tests = vec![
            (0.0, 0.0),
            (-0.0, -0.0),
            (INFINITY, INFINITY),
            (NEG_INFINITY, NEG_INFINITY),
            (f64::MIN, f64::MIN),
            (f64::MAX, f64::MAX),
        ];

        for (input, expected) in tests {
            let num = NumberType::new(input);
            let data = num.marshall().unwrap();

            let mut buf = [0u8; 9];
            buf[0] = TypeMarker::Number as u8;
            buf[1..9].copy_from_slice(&expected.to_be_bytes());

            assert_eq!(data, buf.to_vec());
        }
    }

    #[test]
    fn test_marshall_length() {
        let num = NumberType::new(3.14);
        assert_eq!(num.marshall_length(), 9);
    }

    #[test]
    fn test_unmarshall() {
        let mut data = [0u8; 9];
        data[0] = TypeMarker::Number as u8;
        data[1..9].copy_from_slice(&3.14f64.to_be_bytes());

        let (num, bytes_read) = NumberType::unmarshall(&data).unwrap();

        assert_eq!(bytes_read, 9);
        assert_eq!(num.type_marker, TypeMarker::Number);
        assert!((num.value - 3.14).abs() < EPSILON);
    }

    #[test]
    fn test_unmarshall_special_values() {
        let tests = vec![
            (0.0, 0.0),
            (-0.0, -0.0),
            (INFINITY, INFINITY),
            (NEG_INFINITY, NEG_INFINITY),
            (f64::MIN, f64::MIN),
            (f64::MAX, f64::MAX),
        ];

        for (input, expected) in tests {
            let mut data = [0u8; 9];
            data[0] = TypeMarker::Number as u8;
            data[1..9].copy_from_slice(&input.to_be_bytes());

            let (num, _) = NumberType::unmarshall(&data).unwrap();
            if expected.is_nan() {
                assert!(num.value.is_nan());
            } else {
                assert_eq!(num.value.to_bits(), expected.to_bits());
            }
        }
    }

    #[test]
    fn test_unmarshall_nan() {
        let mut data = [0u8; 9];
        data[0] = TypeMarker::Number as u8;
        data[1..9].copy_from_slice(&NAN.to_be_bytes());

        let (num, _) = NumberType::unmarshall(&data).unwrap();
        assert!(num.value.is_nan());
    }

    #[test]
    fn test_unmarshall_buffer_too_small() {
        let data = [0u8; 8];
        let result = NumberType::unmarshall(&data);
        assert!(matches!(
            result,
            Err(AmfError::BufferTooSmall { want: 9, got: 8 })
        ));
    }

    #[test]
    fn test_unmarshall_invalid_marker() {
        let mut data = [0u8; 9];
        data[0] = TypeMarker::Null as u8; // 错误的类型标记
        data[1..9].copy_from_slice(&3.14f64.to_be_bytes());

        let result = NumberType::unmarshall(&data);
        assert!(matches!(
            result,
            Err(AmfError::TypeMarkerValueMismatch {
                want: 0x00,
                got: 0x05
            })
        ));
    }

    #[test]
    fn test_try_from_slice() {
        let mut data = [0u8; 9];
        data[0] = TypeMarker::Number as u8;
        data[1..9].copy_from_slice(&3.14f64.to_be_bytes());

        let num = NumberType::try_from(&data[..]).unwrap();
        assert!((num.value - 3.14).abs() < EPSILON);
    }

    #[test]
    fn test_deref() {
        let num = NumberType::new(3.14);
        assert!((*num - 3.14).abs() < EPSILON);
    }

    #[test]
    fn test_as_ref() {
        let num = NumberType::new(3.14);
        let value_ref: &f64 = num.as_ref();
        assert!((*value_ref - 3.14).abs() < EPSILON);
    }

    #[test]
    fn test_display() {
        let num = NumberType::new(3.14);
        assert_eq!(format!("{}", num), "3.14");

        let num = NumberType::new(-42.0);
        assert_eq!(format!("{}", num), "-42");

        let num = NumberType::new(INFINITY);
        assert_eq!(format!("{}", num), "inf");

        let num = NumberType::new(NEG_INFINITY);
        assert_eq!(format!("{}", num), "-inf");

        let num = NumberType::new(NAN);
        assert_eq!(format!("{}", num), "NaN");
    }
}
