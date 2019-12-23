use crate::serde::{
    de::{self, Error as DError},
    ser::{self, Error as SError},
};
use std::{error::Error as StdError, fmt, io};

pub enum SerializeError<'a> {
    CustomError(&'a str),
    IoError(io::Error),
}

impl<'a> From<io::Error> for SerializeError<'a> {
    #[inline]
    fn from(self) -> Self {
        Self::IoError(self)
    }
}

impl<'a> fmt::Debug for SerializeError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CustomError(v) => write!(f, "custom error: {:?}", v),
            Self::IoError(e) => write!(f, "{:?}", e),
        }
    }
}

impl<'a> fmt::Display for SerializeError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CustomError(v) => write!(f, "custom error: {}", v),
            Self::IoError(e) => write!(f, "{}", e),
        }
    }
}

impl<'a> StdError for SerializeError<'a> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

impl<'a> SError for SerializeError<'a> {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::CustomError(format!("custom error: {}", msg))
    }
}

pub enum DeserializeError<'a> {
    CustomError(&'a str),
}

impl<'a> fmt::Debug for DeserializeError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CustomError(v) => write!(f, "custom error: {:?}", v),
        }
    }
}

impl<'a> fmt::Display for DeserializeError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CustomError(v) => write!(f, "custom error: {}", v),
        }
    }
}

impl<'a> StdError for DeserializeError<'a> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

impl<'a> SError for DeserializeError<'a> {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self::CustomError(format!("custom error: {}", msg))
    }
}
