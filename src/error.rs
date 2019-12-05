use serde::de::Error as SerdeError;
use std::{error::Error as StdError, fmt};

// TODO: @zerosign, properly use error for defining what error or what not

#[derive(Debug, PartialEq)]
pub enum LiteralError {
    EmptyStr,
    NumberError,
    SyntaxError,
    Unsupported,
    CustomError(String),
}

impl fmt::Display for LiteralError {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NumberError => write!(fmt, "number parse error"),
            Self::EmptyStr => write!(fmt, "empty string given"),
            Self::SyntaxError => write!(fmt, "syntax error"),
            Self::Unsupported => write!(fmt, "unsupported"),
            Self::CustomError(s) => write!(fmt, "{}", s),
        }
    }
}

impl StdError for LiteralError {
    #[inline]
    fn description(&self) -> &str {
        match &*self {
            Self::NumberError => "number parse error",
            Self::EmptyStr => "empty string given",
            Self::SyntaxError => "syntax error",
            Self::Unsupported => "unsupported",
            Self::CustomError(s) => &s,
        }
    }

    #[inline]
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

impl SerdeError for LiteralError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self::CustomError(format!("{}", msg))
    }

    fn missing_field(field: &'static str) -> Self {
        // literal doesn't have missing field
        unimplemented!()
    }
}

#[derive(Debug, PartialEq)]
pub enum ArrayError {
    LiteralError(LiteralError),
    EmptyStr,
    ParseError,
}

impl fmt::Display for ArrayError {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::LiteralError(e) => e.fmt(fmt),
            Self::ParseError => write!(fmt, "parse error"),
            Self::EmptyStr => write!(fmt, "empty string given"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ValueError {
    LiteralError(LiteralError),
    ArrayError(ArrayError),
    EmptyStr,
}

impl fmt::Display for ValueError {
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::LiteralError(e) => e.fmt(fmt),
            Self::ArrayError(e) => e.fmt(fmt),
            Self::EmptyStr => write!(fmt, "empty string given"),
        }
    }
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
    EmptyStr,
    CustomError(String),
    ParseError(ValueError),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::PairError(e) => write!(fmt, "{}", e.description()),
            Error::UnsortedError => write!(fmt, "{}", "pair is unsorted"),
            Error::CustomError(s) => write!(fmt, "{}", s),
            Error::ParseError(e) => write!(fmt, "{}", e),
            Error::EmptyStr => write!(fmt, "{}", "empty string"),
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
            Error::ParseError(e) => "parse error",
            Error::EmptyStr => "empty string",
        }
    }

    #[inline]
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

impl SerdeError for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::CustomError(format!("{}", msg))
    }

    fn missing_field(field: &'static str) -> Self {
        Error::PairError(PairError::IncompletePair(String::from(field)))
    }
}
