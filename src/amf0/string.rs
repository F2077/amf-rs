use crate::amf0::type_marker::TypeMarker;
use crate::amf0::utf8::AmfUtf8;
use crate::errors::AmfError;
use crate::traits::{Marshall, MarshallLength, Unmarshall};
use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::io;
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AmfUtf8ValuedType<const LENGTH_BYTE_WIDTH: usize, const TYPE_MARKER: u8> {
    inner: AmfUtf8<LENGTH_BYTE_WIDTH>,
}

impl<const LENGTH_BYTE_WIDTH: usize, const TYPE_MARKER: u8>
    AmfUtf8ValuedType<LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    pub fn new(inner: AmfUtf8<LENGTH_BYTE_WIDTH>) -> Self {
        Self { inner }
    }
}

impl<const LENGTH_BYTE_WIDTH: usize, const TYPE_MARKER: u8> Marshall
    for AmfUtf8ValuedType<LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    fn marshall(&self) -> Result<Vec<u8>, AmfError> {
        let mut vec = Vec::with_capacity(self.marshall_length());
        vec.push(TYPE_MARKER);
        let inner_vec = self.inner.marshall()?;
        vec.extend_from_slice(inner_vec.as_slice());
        Ok(vec)
    }
}

impl<const LENGTH_BYTE_WIDTH: usize, const TYPE_MARKER: u8> MarshallLength
    for AmfUtf8ValuedType<LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    fn marshall_length(&self) -> usize {
        1 + self.inner.marshall_length()
    }
}

impl<const LENGTH_BYTE_WIDTH: usize, const TYPE_MARKER: u8> Unmarshall
    for AmfUtf8ValuedType<LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    fn unmarshall(buf: &[u8]) -> Result<(Self, usize), AmfError> {
        let required_size = 1 + LENGTH_BYTE_WIDTH;
        if buf.len() < required_size {
            return Err(AmfError::BufferTooSmall {
                want: required_size,
                got: buf.len(),
            });
        }

        if buf[0] != TYPE_MARKER {
            return Err(AmfError::TypeMarkerValueMismatch {
                want: TYPE_MARKER,
                got: buf[0],
            });
        }
        let inner = AmfUtf8::unmarshall(&buf[1..])?;
        Ok((Self::new(inner.0), 1 + inner.1))
    }
}

impl<const LENGTH_BYTE_WIDTH: usize, const TYPE_MARKER: u8> TryFrom<AmfUtf8<LENGTH_BYTE_WIDTH>>
    for AmfUtf8ValuedType<LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    type Error = io::Error;

    fn try_from(value: AmfUtf8<LENGTH_BYTE_WIDTH>) -> Result<Self, Self::Error> {
        Ok(Self::new(value))
    }
}

impl<const LENGTH_BYTE_WIDTH: usize, const TYPE_MARKER: u8> AsRef<AmfUtf8<LENGTH_BYTE_WIDTH>>
    for AmfUtf8ValuedType<LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    fn as_ref(&self) -> &AmfUtf8<LENGTH_BYTE_WIDTH> {
        &self.inner
    }
}

impl<const LENGTH_BYTE_WIDTH: usize, const TYPE_MARKER: u8> Deref
    for AmfUtf8ValuedType<LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    type Target = AmfUtf8<LENGTH_BYTE_WIDTH>;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<const LENGTH_BYTE_WIDTH: usize, const TYPE_MARKER: u8> Borrow<AmfUtf8<LENGTH_BYTE_WIDTH>>
    for AmfUtf8ValuedType<LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    fn borrow(&self) -> &AmfUtf8<LENGTH_BYTE_WIDTH> {
        self.as_ref()
    }
}

impl<const LENGTH_BYTE_WIDTH: usize, const TYPE_MARKER: u8> Display
    for AmfUtf8ValuedType<LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl<const LENGTH_BYTE_WIDTH: usize, const TYPE_MARKER: u8> Default
    for AmfUtf8ValuedType<LENGTH_BYTE_WIDTH, TYPE_MARKER>
{
    fn default() -> Self {
        Self::new(AmfUtf8::<LENGTH_BYTE_WIDTH>::default())
    }
}

//	All strings in AMF are encoded using UTF-8; however, the byte-length header format
//	may vary. The AMF 0 String type uses the standard byte-length header (i.e. U16). For
//	long Strings that require more than 65535 bytes to encode in UTF-8, the AMF 0 Long
//	String type should be used.
pub type StringType = AmfUtf8ValuedType<2, { TypeMarker::String as u8 }>;

//	A long string is used in AMF 0 to encode strings that would occupy more than 65535
//	bytes when UTF-8 encoded. The byte-length header of the UTF-8 encoded string is a 32-
//	bit integer instead of the regular 16-bit integer.
pub type LongStringType = AmfUtf8ValuedType<4, { TypeMarker::LongString as u8 }>;
