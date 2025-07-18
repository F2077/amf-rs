use crate::errors::AmfError;

pub trait TryIntoBytes {
    fn try_into_bytes(&self) -> Result<&[u8], AmfError>;
}
pub trait TryFromBytes: Sized {
    fn try_from_bytes(buf: &[u8]) -> Result<(Self, usize), AmfError>;
}
pub trait Length {
    fn length(&self) -> usize;
}
