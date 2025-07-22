use crate::errors::AmfError;
use crate::traits::{Marshall, MarshallLength, Unmarshall};
use std::fmt::Display;

//	If a type cannot be serialized a special unsupported marker can be used in place of the
//	type. Some endpoints may throw an error on encountering this type marker. No further
//	information is encoded for this type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct UnsupportedType {}

impl Marshall for UnsupportedType {
    fn marshall(&self) -> Result<Vec<u8>, AmfError> {
        panic!("unsupported")
    }
}

impl MarshallLength for UnsupportedType {
    fn marshall_length(&self) -> usize {
        panic!("unsupported")
    }
}

impl Unmarshall for UnsupportedType {
    fn unmarshall(_buf: &[u8]) -> Result<(Self, usize), AmfError> {
        panic!("unsupported")
    }
}

// 实现 rust 惯用语("idiom") 方便用户使用

impl Display for UnsupportedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unsupported type: {}", stringify!(self))
    }
}

// 类型别名

//	This type is not supported and is reserved for future use.
pub type MovieClipType = UnsupportedType;

//	This type is not supported and is reserved for future use.
pub type RecordsetType = UnsupportedType;

// 以下这些类型大概率在实际应用中用不到，所以暂时不实现
pub type ReferenceType = UnsupportedType;
pub type StrictArrayType = UnsupportedType;
pub type DateType = UnsupportedType;
pub type XmlDocumentType = UnsupportedType;
pub type TypedObjectType = UnsupportedType;
