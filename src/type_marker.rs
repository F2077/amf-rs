use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeMarkerError {
    value: u8,
}

impl fmt::Display for TypeMarkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unknown TypeMarker value: 0x{:02X}", self.value)
    }
}

impl std::error::Error for TypeMarkerError {}

// 实现从 u8 到 TypeMarker 的安全转换
impl TryFrom<u8> for TypeMarker {
    type Error = TypeMarkerError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Number),
            0x01 => Ok(Self::Boolean),
            0x02 => Ok(Self::String),
            0x03 => Ok(Self::Object),
            0x04 => Ok(Self::MovieClip),
            0x05 => Ok(Self::Null),
            0x06 => Ok(Self::Undefined),
            0x07 => Ok(Self::Reference),
            0x08 => Ok(Self::EcmaArray),
            0x09 => Ok(Self::ObjectEnd),
            0x0A => Ok(Self::StrictArray),
            0x0B => Ok(Self::Date),
            0x0C => Ok(Self::LongString),
            0x0D => Ok(Self::Unsupported),
            0x0E => Ok(Self::Recordset),
            0x0F => Ok(Self::XmlDocument),
            0x10 => Ok(Self::TypedObject),
            other => Err(TypeMarkerError { value: other }),
        }
    }
}

// 实现从 TypeMarker 到 u8 的转换
impl From<TypeMarker> for u8 {
    fn from(marker: TypeMarker) -> Self {
        marker as u8
    }
}

// 实现 Display trait 用于友好打印
impl std::fmt::Display for TypeMarker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
