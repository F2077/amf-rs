use crate::errors::AmfError;
use crate::traits::{Marshall, MarshallLength, Unmarshall};

//	If a type cannot be serialized a special unsupported marker can be used in place of the
//	type. Some endpoints may throw an error on encountering this type marker. No further
//	information is encoded for this type.
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
    fn unmarshall(buf: &[u8]) -> Result<(Self, usize), AmfError> {
        panic!("unsupported")
    }
}

//	This type is not supported and is reserved for future use.
pub type MovieClipType = UnsupportedType;

//	This type is not supported and is reserved for future use.
pub type RecordSetType = UnsupportedType;

// 以下这些类型大概率在实际应用中用不到，所以暂时不实现
pub type ReferenceType = UnsupportedType;
pub type StrictArrayType = UnsupportedType;
pub type DateType = UnsupportedType;
pub type XMLDocumentType = UnsupportedType;
pub type TypedObjectType = UnsupportedType;
