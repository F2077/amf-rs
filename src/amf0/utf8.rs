use crate::errors::AmfError;
use crate::traits::{FromBytes, Length, ToBytes};
use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AmfUtf8<const W: usize> {
    inner: String,
}

impl<const W: usize> AmfUtf8<W> {
    pub fn new(inner: &str) -> Result<Self, AmfError> {
        debug_assert!(W == 2 || W == 4);
        let len = inner.len();
        if (W == 2 && len > u16::MAX as usize) || (W == 4 && len > u32::MAX as usize) {
            return Err(AmfError::StringTooLong { max: W, got: len });
        }
        Ok(Self {
            inner: inner.to_string(),
        })
    }
}

impl<const W: usize> ToBytes for AmfUtf8<W> {
    fn to_bytes(&self) -> Result<Vec<u8>, AmfError> {
        debug_assert!(W == 2 || W == 4);
        let mut buf = Vec::with_capacity(self.bytes_size());
        let length_buf = if W == 2 {
            (self.inner.len() as u16).to_be_bytes()
        } else if W == 4 {
            (self.inner.len() as u32).to_be_bytes()
        };
        buf.extend_from_slice(&length_buf);
        buf.extend_from_slice(self.inner.as_bytes());
        Ok(buf)
    }
}

impl<const W: usize> Length for AmfUtf8<W> {
    fn length(&self) -> usize {
        debug_assert!(W == 2 || W == 4);
        W + self.inner.len()
    }
}

impl<const W: usize> FromBytes for AmfUtf8<W> {
    fn from_bytes(buf: &[u8]) -> Result<(Self, usize), AmfError> {
        debug_assert!(W == 2 || W == 4);
        let length = if W == 2 {
            u16::from_be_bytes(
                buf[0..2]
                    .iter()
                    .try_into()?
                    .map_err(|_| AmfError::Custom("failed to parse length (u16) from buf")),
            )
        } else {
            u32::from_be_bytes(
                buf[0..4]
                    .iter()
                    .try_into()?
                    .map_err(|_| AmfError::Custom("failed to parse length (u32) from buf")),
            )
        };

        let start = W;
        let end = start + length;
        if buf.len() < end {
            return Err(AmfError::BufferTooSmall {
                expected: end,
                got: buf.len(),
            });
        }
        let value = std::str::from_utf8(&buf[start..end]).map_err(|e| AmfError::Io(e))?;
        Ok((end, value))
    }
}

// ------------- 实现 rust 惯用语("idiom") 方便用户使用 -------------

impl<'a, const W: usize> TryInto<&'a [u8]> for AmfUtf8<W> {
    type Error = AmfError;

    fn try_into(self) -> Result<&'a [u8], Self::Error> {
        self.to_bytes().map(|v| &v[..])
    }
}

impl<const W: usize> TryInto<Vec<u8>> for AmfUtf8<W> {
    type Error = AmfError;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        self.to_bytes()
    }
}

impl<const W: usize> TryFrom<&[u8]> for AmfUtf8<W> {
    type Error = AmfError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        Self::from_bytes(value).map(|(v, _)| v)
    }
}

impl<const W: usize> TryFrom<Vec<u8>> for AmfUtf8<W> {
    type Error = AmfError;
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::from_bytes(&value).map(|(v, _)| v)
    }
}

impl<const W: usize> Display for AmfUtf8<W> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}
impl<const W: usize> AsRef<str> for AmfUtf8<W> {
    fn as_ref(&self) -> &str {
        self.inner.as_ref()
    }
}
impl<const W: usize> Deref for AmfUtf8<W> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        Self::as_ref(self)
    }
}
impl<const W: usize> Borrow<str> for AmfUtf8<W> {
    fn borrow(&self) -> &str {
        Self::as_ref(self)
    }
}
