use std::fmt;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl Display for TypeMarker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<TypeMarker> for u8 {
    fn from(marker: TypeMarker) -> Self {
        marker as u8
    }
}
