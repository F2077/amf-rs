use crate::errors::AmfError;

pub trait Length {
    fn length(&self) -> usize;
}

pub trait ToBytes {
    fn to_bytes(&self) -> Result<Vec<u8>, AmfError>;
}
pub trait FromBytes: Sized {
    fn from_bytes(buf: &[u8]) -> Result<(Self, usize), AmfError>;
}
