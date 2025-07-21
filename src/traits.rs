use crate::errors::AmfError;

pub trait Marshall {
    fn marshall(&self) -> Result<Vec<u8>, AmfError>;
}

pub trait MarshallLength {
    fn marshall_length(&self) -> usize;
}

pub trait Unmarshall: Sized {
    fn unmarshall(buf: &[u8]) -> Result<(Self, usize), AmfError>;
}
