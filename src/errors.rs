use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::io;

#[derive(Debug)]
pub enum AmfError {
    BufferTooSmall { expected: usize, got: usize },
    StringTooLong { max: usize, got: usize },
    Custom(String),
    Io(io::Error),
}

impl Display for AmfError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AmfError::BufferTooSmall { expected, got } => {
                write!(
                    f,
                    "buffer too small: expected {} bytes, got {}",
                    expected, got
                )
            }
            AmfError::StringTooLong { max, got } => {
                write!(f, "string too long: max {}, got {}", max, got)
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
