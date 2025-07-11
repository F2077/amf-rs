use std::io;

pub trait ToBytes {
    fn to_bytes(&self) -> io::Result<Vec<u8>>;
    fn bytes_size(&self) -> usize;
    fn write_bytes_to(&self, buf: &mut [u8]) -> io::Result<usize>;
}
pub trait FromBytes<'a>: Sized {
    fn from_bytes_owned(buf: &[u8]) -> io::Result<Self>;
    fn from_bytes_borrowed(buf: &'a [u8]) -> io::Result<Self>;
}
