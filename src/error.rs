use crate::serde::{
    ser::{Error as SError},
};
use std::{error::Error as StdError, fmt, io};

pub enum SerializeError {
    /// dedicated for custom error in user space
    CustomError(String),
    /// dedicated for std io::Error wrapper
    IoError(io::Error),
    /// dedicated for unknown state error when doing serializing
    /// in either both `crate::ser::MapFlow` or `crate::ser::SeqFlow`
    StateError,
}

impl From<io::Error> for SerializeError {
    #[inline]
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

impl fmt::Debug for SerializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CustomError(v) => write!(f, "custom error: {:?}", v),
            Self::IoError(e) => write!(f, "{:?}", e),
            Self::StateError => write!(f, "{}", "StateError"),
        }
    }
}

impl fmt::Display for SerializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CustomError(v) => write!(f, "custom error: {}", v),
            Self::IoError(e) => write!(f, "{}", e),
            Self::StateError => write!(f, "{}", "StateError"),
        }
    }
}

impl StdError for SerializeError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

impl SError for SerializeError {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::CustomError(format!("custom error: {}", msg))
    }
}

pub enum DeserializeError {
    CustomError(String),
}

impl fmt::Debug for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CustomError(v) => write!(f, "custom error: {:?}", v),
        }
    }
}

impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CustomError(v) => write!(f, "custom error: {}", v),
        }
    }
}

impl StdError for DeserializeError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

impl SError for DeserializeError {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::CustomError(format!("custom error: {}", msg))
    }
}
