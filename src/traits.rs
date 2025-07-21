use crate::errors::AmfError;

pub(crate) trait Marshall {
    fn marshall(&self) -> Result<Vec<u8>, AmfError>;
}

pub(crate) trait MarshallLength {
    fn marshall_length(&self) -> usize;
}

pub(crate) trait Unmarshall: Sized {
    fn unmarshall(buf: &[u8]) -> Result<(Self, usize), AmfError>;
}
