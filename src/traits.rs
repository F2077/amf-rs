use std::io;

pub trait ToBytes {
    fn to_bytes(&self) -> io::Result<Vec<u8>>;
    fn bytes_size(&self) -> usize;
    fn write_bytes_to(&self, buf: &mut [u8]) -> io::Result<usize>;
}
pub trait FromBytes: Sized {
    fn from_bytes(buf: &[u8]) -> io::Result<Self>;
}

pub trait FromBytesRef<'a>: Sized {
    fn from_bytes_ref(buf: &'a [u8]) -> io::Result<Self>;
}
