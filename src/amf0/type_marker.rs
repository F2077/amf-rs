use crate::errors::AmfError;
use std::fmt;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)] // 指定 TypeMarker 类型为 u8 类型(指定枚举类型底层判别值的存储类型)
pub enum TypeMarker {
    Number = 0x00,
    Boolean = 0x01,
    String = 0x02,
    Object = 0x03,
    MovieClip = 0x04, // reserved, not supported
    Null = 0x05,
    Undefined = 0x06,
    Reference = 0x07,
    EcmaArray = 0x08,
    ObjectEnd = 0x09,
    StrictArray = 0x0A,
    Date = 0x0B,
    LongString = 0x0C,
    Unsupported = 0x0D,
    Recordset = 0x0E, // reserved, not supported
    XmlDocument = 0x0F,
    TypedObject = 0x10,
}

impl TryFrom<u8> for TypeMarker {
    type Error = AmfError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(TypeMarker::Number),
            0x01 => Ok(TypeMarker::Boolean),
            0x02 => Ok(TypeMarker::String),
            0x03 => Ok(TypeMarker::Object),
            0x04 => Ok(TypeMarker::MovieClip),
            0x05 => Ok(TypeMarker::Null),
            0x06 => Ok(TypeMarker::Undefined),
            0x07 => Ok(TypeMarker::Reference),
            0x08 => Ok(TypeMarker::EcmaArray),
            0x09 => Ok(TypeMarker::ObjectEnd),
            0x0A => Ok(TypeMarker::StrictArray),
            0x0B => Ok(TypeMarker::Date),
            0x0C => Ok(TypeMarker::LongString),
            0x0D => Ok(TypeMarker::Unsupported),
            0x0E => Ok(TypeMarker::Recordset),
            0x0F => Ok(TypeMarker::XmlDocument),
            0x10 => Ok(TypeMarker::TypedObject),
            v => Err(AmfError::Custom(format!(
                "Invalid type marker value: {:?}",
                v
            ))),
        }
    }
}

impl Display for TypeMarker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
