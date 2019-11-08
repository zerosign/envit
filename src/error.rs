#[cfg(feature = "envit_serde")]
use serde::de::Error as SerdeError;
use std::{error::Error as StdError, fmt};

#[derive(Debug, PartialEq)]
pub enum LiteralError {
    EmptyStr,
    NumberError,
}

#[derive(Debug, PartialEq)]
pub enum ArrayError {
    LiteralError(LiteralError),
    EmptyStr,
    ParseError,
}

#[derive(Debug, PartialEq)]
pub enum ValueError {
    LiteralError(LiteralError),
    ArrayError(ArrayError),
    EmptyStr,
}

#[derive(Debug, PartialEq)]
pub enum PairError {
    EmptyPair,
    IncompletePair(String),
    SizeError,
}

impl fmt::Display for PairError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PairError::EmptyPair => write!(fmt, "pair format empty"),
            PairError::IncompletePair(f) => write!(fmt, "no value for key {}", f),
            PairError::SizeError => write!(
                fmt,
                "the size of the pair of key & value should be exactly 2"
            ),
        }
    }
}

impl StdError for PairError {
    #[inline]
    fn description(&self) -> &str {
        match *self {
            PairError::EmptyPair => "pair format empty",
            PairError::IncompletePair(ref f) => f,
            PairError::SizeError => "the size of the pair of key & value should be exactly 2",
        }
    }

    #[inline]
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    PairError(PairError),
    UnsortedError,
    CustomError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::PairError(e) => write!(fmt, "{}", e.description()),
            Error::UnsortedError => write!(fmt, "{}", "pair is unsorted"),
            Error::CustomError(s) => write!(fmt, "{}", s),
        }
    }
}

impl StdError for Error {
    #[inline]
    fn description(&self) -> &str {
        match self {
            Error::PairError(e) => e.description(),
            Error::UnsortedError => "pair is unsorted",
            Error::CustomError(s) => s.as_ref(),
        }
    }

    #[inline]
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

#[cfg(feature = "envit_serde")]
impl SerdeError for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::CustomError(format!("{}", msg))
    }

    fn missing_field(field: &'static str) -> Self {
        Error::PairError(PairError::IncompletePair(String::from(field)))
    }
}
