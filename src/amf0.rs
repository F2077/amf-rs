use crate::traits::{FromBytes, ToBytes};
use crate::type_marker::TypeMarker;
use crate::utf8;
use crate::utf8::Utf8;
use indexmap::IndexMap;
use std::io;

pub trait AmfType: ToBytes + FromBytes {}

// An AMF 0 Number type is used to encode an ActionScript Number.
// The data following a Number type marker is always an 8 byte IEEE-754 double precision floating point value in network byte order (sign bit in low memory).
#[derive(Debug, PartialEq)]
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

impl ToBytes for NumberType {
    fn to_bytes(&self) -> io::Result<Vec<u8>> {
        debug_assert!(self.type_marker == TypeMarker::Number);
        let mut vec = Vec::with_capacity(self.bytes_size());
        vec.push(self.type_marker as u8);
        vec.extend_from_slice(&self.value.to_be_bytes());
        Ok(vec)
    }

    fn bytes_size(&self) -> usize {
        1 + 8 // 1 byte for type marker + 8 bytes for value
    }

    fn write_bytes_to(&self, buf: &mut [u8]) -> io::Result<usize> {
        debug_assert!(self.type_marker == TypeMarker::Number);
        let required_size = self.bytes_size();
        if buf.len() < required_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Buffer is too small, need at least {} bytes", required_size),
            ));
        }

        buf[0] = self.type_marker as u8;
        buf[1..9].copy_from_slice(&self.value.to_be_bytes());
        Ok(9)
    }
}

impl FromBytes for NumberType {
    fn from_bytes(buf: &[u8]) -> io::Result<Self> {
        if buf.len() < 9 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Buffer is too small, need at least 9 bytes",
            ));
        }
        let type_marker = TypeMarker::try_from(buf[0])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if type_marker != TypeMarker::Number {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid type marker, expected Number, got {:?}",
                    type_marker
                ),
            ));
        }
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

impl BooleanType {
    pub fn new(value: bool) -> Self {
        Self {
            type_marker: TypeMarker::Boolean,
            value,
        }
    }
}

impl ToBytes for BooleanType {
    fn to_bytes(&self) -> io::Result<Vec<u8>> {
        debug_assert!(self.type_marker == TypeMarker::Boolean);
        let mut vec = Vec::with_capacity(self.bytes_size());
        vec.push(self.type_marker as u8);
        vec.push(self.value as u8);
        Ok(vec)
    }

    fn bytes_size(&self) -> usize {
        1 + 1 // 1 byte for type marker + 1 byte for value
    }

    fn write_bytes_to(&self, buf: &mut [u8]) -> io::Result<usize> {
        debug_assert!(self.type_marker == TypeMarker::Boolean);
        let required_size = self.bytes_size();
        if buf.len() < required_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Buffer is too small, need at least {} bytes", required_size),
            ));
        }
        buf[0] = self.type_marker as u8;
        buf[1] = self.value as u8;
        Ok(2)
    }
}

impl FromBytes for BooleanType {
    fn from_bytes(buf: &[u8]) -> io::Result<Self> {
        if buf.len() < 2 {
            return Err(io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Buffer is too small, need at least 2 bytes",
            ));
        }
        let type_marker = TypeMarker::try_from(buf[0])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if type_marker != TypeMarker::Boolean {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid type marker, expected Boolean, got {:?}",
                    type_marker
                ),
            ));
        }
        let value = buf[1] != 0;
        Ok(Self { type_marker, value })
    }
}

//	All strings in AMF are encoded using UTF-8; however, the byte-length header format
//	may vary. The AMF 0 String type uses the standard byte-length header (i.e. U16). For
//	long Strings that require more than 65535 bytes to encode in UTF-8, the AMF 0 Long
//	String type should be used.
#[derive(Debug, PartialEq)]
pub struct StringType<'a> {
    type_marker: TypeMarker,
    value: Utf8<'a>,
}

impl<'a> StringType<'a> {
    pub fn new_owned(value: String) -> Result<Self, io::Error> {
        let value_utf8 = Utf8::new_owned(value)?;
        Ok(Self {
            type_marker: TypeMarker::String,
            value: value_utf8,
        })
    }

    pub fn new_borrowed(value: &'a str) -> Result<Self, io::Error> {
        let value_utf8 = Utf8::new_borrowed(value)?;
        Ok(Self {
            type_marker: TypeMarker::String,
            value: value_utf8,
        })
    }
}

impl<'a> ToBytes for StringType<'a> {
    fn to_bytes(&self) -> io::Result<Vec<u8>> {
        debug_assert!(self.type_marker == TypeMarker::String);
        let mut vec = Vec::with_capacity(self.bytes_size());
        vec.push(self.type_marker as u8);
        let _ = self.value.write_bytes_to(&mut vec[1..])?;
        Ok(vec)
    }

    fn bytes_size(&self) -> usize {
        1 + self.value.bytes_size()
    }

    fn write_bytes_to(&self, buf: &mut [u8]) -> io::Result<usize> {
        debug_assert!(self.type_marker == TypeMarker::String);
        let required_size = self.bytes_size();
        if buf.len() < required_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Buffer is too small, need at least {} bytes", required_size),
            ));
        }
        buf[0] = self.type_marker as u8;
        let n = self.value.write_bytes_to(&mut buf[1..])?;

        Ok(1 + n)
    }
}

#[derive(Debug, PartialEq)]
pub struct ObjectEndType {
    empty: Utf8<'static>,
    type_marker: TypeMarker,
}

impl ObjectEndType {
    pub fn new() -> Self {
        Self {
            empty: utf8::EMPTY_UTF8,
            type_marker: TypeMarker::ObjectEnd,
        }
    }
}

impl ToBytes for ObjectEndType {
    fn to_bytes(&self) -> io::Result<Vec<u8>> {
        debug_assert!(self.type_marker == TypeMarker::ObjectEnd);
        let mut vec = Vec::with_capacity(self.bytes_size());
        let _ = self.empty.write_bytes_to(&mut vec[0..2])?;
        vec.push(self.type_marker as u8);
        Ok(vec)
    }

    fn bytes_size(&self) -> usize {
        2 + 1
    }

    fn write_bytes_to(&self, buf: &mut [u8]) -> io::Result<usize> {
        debug_assert!(self.type_marker == TypeMarker::ObjectEnd);
        let required_size = self.bytes_size();
        if buf.len() < required_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Buffer is too small, need at least {} bytes", required_size),
            ));
        }
        let _ = self.empty.write_bytes_to(&mut buf[0..2])?;
        buf[2] = self.type_marker as u8;
        Ok(3)
    }
}

impl FromBytes for ObjectEndType {
    fn from_bytes(buf: &[u8]) -> io::Result<Self> {
        if buf.len() < 3 {
            return Err(io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Buffer is too small, need at least 3 bytes",
            ));
        }
        let empty = Utf8::from_bytes(&buf[0..2])?;
        let type_marker = TypeMarker::try_from(buf[2])
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if type_marker != TypeMarker::ObjectEnd {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid type marker, expected ObjectEnd, got {:?}",
                    type_marker
                ),
            ));
        }
        Ok(Self { empty, type_marker })
    }
}

pub const OBJECT_END: ObjectEndType = ObjectEndType {
    empty: utf8::EMPTY_UTF8,
    type_marker: TypeMarker::ObjectEnd,
};

#[derive(Debug)]
pub struct ObjectType<'a, T: AmfType> {
    type_marker: TypeMarker,
    properties: IndexMap<Utf8<'a>, T>,
    object_end: ObjectEndType,
}

impl<'a, T: AmfType> ObjectType<'a, T> {
    pub fn new(properties: IndexMap<Utf8<'a>, T>) -> Self {
        Self {
            type_marker: TypeMarker::Object,
            properties,
            object_end: OBJECT_END,
        }
    }
}

impl<'a, T: AmfType> ToBytes for ObjectType<'a, T> {
    fn to_bytes(&self) -> io::Result<Vec<u8>> {
        debug_assert!(self.type_marker == TypeMarker::Object);
        let mut vec = Vec::with_capacity(self.bytes_size());
        vec.push(self.type_marker as u8);
        self.properties
            .iter()
            .try_for_each(|(k, v)| -> io::Result<()> {
                let k_vec = k
                    .to_bytes()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                vec.extend_from_slice(&k_vec);
                let v_vec = v
                    .to_bytes()
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                vec.extend_from_slice(&v_vec);
                Ok(())
            })?;

        let object_end_vec = self.object_end.to_bytes()?;
        vec.extend_from_slice(&object_end_vec);

        Ok(vec)
    }

    fn bytes_size(&self) -> usize {
        let mut size = 0;
        size += 1; // type marker length
        let properties_bytes_size: usize = self
            .properties
            .iter()
            .map(|(k, v)| k.bytes_size() + v.bytes_size())
            .sum();
        size += properties_bytes_size;
        size += self.object_end.bytes_size();
        size
    }

    fn write_bytes_to(&self, buf: &mut [u8]) -> io::Result<usize> {
        debug_assert!(self.type_marker == TypeMarker::Object);
        let required_size = self.bytes_size();
        if buf.len() < required_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Buffer is too small, need at least {} bytes", required_size),
            ));
        }

        buf[0] = self.type_marker as u8;
        let mut offset = 1;
        for (k, v) in &self.properties {
            k.write_bytes_to(&mut buf[offset..offset + k.bytes_size()])?;
            offset += k.bytes_size();
            v.write_bytes_to(&mut buf[offset..offset + v.bytes_size()])?;
            offset += v.bytes_size();
        }

        Ok(offset)
    }
}
