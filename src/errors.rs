use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::io;

#[derive(Debug)]
pub enum AmfError {
    BufferTooSmall { want: usize, got: usize },
    StringTooLong { max: usize, got: usize },
    InvalidUtf8(std::str::Utf8Error),
    TypeMarkerValueMismatch { want: u8, got: u8 },
    Custom(String),
    Io(io::Error),
}

impl Display for AmfError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AmfError::BufferTooSmall { want, got } => {
                write!(f, "Buffer too small: want {} bytes, got {}", want, got)
            }
            AmfError::StringTooLong { max, got } => {
                write!(f, "String too long: max {}, got {}", max, got)
            }
            AmfError::InvalidUtf8(err) => {
                write!(f, "{}", err)
            }
            AmfError::TypeMarkerValueMismatch { want, got } => {
                write!(f, "Type marker value mismatch: want {}, got {}", want, got)
            }
            AmfError::Custom(msg) => {
                write!(f, "{}", msg)
            }
            AmfError::Io(err) => {
                write!(f, "{}", err)
            }
        }
    }
}

// 用来支持 ? 操作符
impl From<io::Error> for AmfError {
    fn from(value: io::Error) -> Self {
        AmfError::Io(value)
    }
}

impl Error for AmfError {
    // 覆写是为了让错误链可以正常工作
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AmfError::Io(err) => Some(err),
            _ => None,
        }
    }
}
