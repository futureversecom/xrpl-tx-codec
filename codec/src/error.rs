use alloc::string::String;
use core::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    OutOfRange(String),
    InvalidData(String),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfRange(s) => write!(f, "Value is out of range: {}", s),
            Self::InvalidData(s) => write!(f, "Value not valid in the given context: {}", s),
        }
    }
}
