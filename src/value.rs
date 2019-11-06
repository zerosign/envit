use crate::error::{PrimitiveError, ValueError};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Number {
    Integer(i64),
    Double(f64),
}

#[derive(Debug, PartialEq)]
pub enum Primitive {
    Number(Number),
    String(String),
    Bool(bool),
}

#[derive(Debug, PartialEq)]
enum State {
    String,
    Int,
    Double,
}

impl Parser for Primitive {
    type Item = Self;
    type Error = Error;

    /// Parse raw &str into Primitive value.
    ///
    /// 1. Simple case (simple lookup)
    ///    - empty str
    ///    - quoted str
    ///    - boolean (true | false)
    ///
    /// 2. scan phase
    ///    - raw -> int | double | string .
    ///    - int -> double | string .
    ///    - double -> double | string .
    ///    - string -> string .
    ///
    pub fn parse(raw: &str) -> Result<Self::Item, Self::Error> {
        let raw = raw.trim();
        // empty string or quoted string or boolean
        if raw.is_empty() {
            Err(Self::Error::EmptyErr)
        } else if raw.starts_with('"') && raw.ends_with('"') {
            Ok(Primitive::String(String::from(&raw[1..raw.len() - 1])))
        } else if raw == "true" {
            Ok(Primitive::Bool(true))
        } else if raw == "false" {
            Ok(Primitive::Bool(false))
        } else {
            // need scan phase (diff integer, double and string)
            // raw -> int | double | string .
            // int -> double | string .
            // double -> double | string .
            // string -> string .
            raw.iter().fold(None, |state, item| {
                match state {
                    // int -> double | string
                    Some(State::Int) => if char::is_digit(ch) {
                        Some(State::Int)
                    } else if ch == '.' {
                        Some(State::Double)
                    } else {
                        Some(State::String)
                    },
                    // double -> double | string
                    Some(State::Double) => if char::is_digit(ch) {
                        Some(State::Double)
                    } else {
                        Some(State::String)
                    },
                    // string -> string | bool .
                    Some(State::String) => Some(State::String),
                    None => if char::is_digit(ch) {
                        Some(State::Int)
                    } else {
                        Some(State::String)
                    }
                }
            }) match {
                Some(State::Int) => raw.parse::<i64>()
                    .map_err(|_| Self::Error::NumberError)
                    .map(|v| Primitive::Number(Number::Integer(v))),
                Some(State::Double) => raw.parse::<f64>()
                    .map_err(|_| Self::Error::NumberError)
                    .map(|v| Primitive::Number(Number::Double(v))),
                Some(State::String) => Ok(Primitive::String(raw)),
                _ => Err(Self::Error::EmptyStr)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct LinearArray(Vec<Primitive>);

impl LinearArray {
    pub fn as_vec(&self) -> Vec<Primitive> {
        self.inner.cloned()
    }
}

///
/// recursively find index where separator `sep` is located for quoted str.
///
pub(crate) fn lookup_quoted_sep(raw: &str, sep: char) -> Option<usize> {
    // found quoted str
    match raw.find(sep) {
        Some(idx) => if &raw[idx-1..idx] == "\\" {
            // escaped str, skip
            lookup_quoted_sep(&raw[idx+1..raw.len()], sep).map(|v| v + idx)
        } else {
            Some(idx)
        },
        _ => None
    }
}

///
/// recursively find index where array separator `sep` is located in array
/// that may contains quoted str.
///
pub(crate) fn lookup_array_sep(raw: &str, sep: char) -> Option<usize> {
    if raw.starts_with('"') {
        match lookup_quoted_sep(&raw[1..raw.len()], '"') {
            Some(idx) => lookup_quoted_sep(&raw[idx..raw.len()], sep).map(|v| idx + v),
            _ => None,
        }
    } else {
        raw.find(sep)
    }
}

impl Parser for LinearArray {
    type Item = Value;
    type Error = ValueError;

    fn parse(raw: &str) -> Result<Self::Item, Self::Error> {
        let raw = raw.trim();

        if raw.is_empty() {
            Err(Self::Error::EmptyStr)
        } else if raw.starts_with('[') && raw.ends_with(']') { // detect array value
            // if its an array split based on ',' (careful of quoted str)
            let slices = &raw[1..raw.len()-1].trim();
            let data = vec![];
            let cursor = slices;

            //
            // quote escape by using lookup separator function.
            // the idea is actively looking for separator but also
            // do escape quotation by skipping quoted str.
            //
            // when idx where separator found, slice the str in [idx + 1]
            //
            while let Some(idx) = lookup_array_sep(cursor, ',') {
                let el = Primitive::parse(&cursor[0..idx]).map_err(Self::Error::PrimitiveError)?;
                data.push_back(el);
                cursor = cursor[idx+1..cursor.len()];
            }

            Ok(Value::Array(Self {
                inner: data
            }))
        } else {
            Primitive::parse(raw).map_err(Self::Error::PrimitiveError)
        }
    }
}

///
/// Value representation of environment variable.
///
/// notes: In environment variable there is no such thing as array with
/// objects then our array types are flat & linear.
///
#[derive(Debug, PartialEq)]
pub enum Value {
    Primitive(Primitive),
    /// in here array only can holds [`Primitive`](Primitive) values.
    /// no array in array or object in array allowed.
    Array(LinearArray),
    Object(HashMap<String, Value>),
}

impl Value {

    pub fn as_str() -> Option<&str> {
        match self {
            Self::Primitive(Primitive::String(s)) => Some(s.into()),
            _ => None
        }
    }

    pub fn as_map() -> Option<HashMap<String, Value>> {
        match self {
            Self::Object(inner) => inner.cloned(),
            _ => None
        }
    }

    pub fn as_vec() -> Option<Vec<Primitive>> {
        match self {
            Self::Array(s) => Some(s.as_vec()),
            _ => None
        }
    }

    pub fn as_double() -> Option<f64> {
        match self {
            Self::Primitive(Primitive::Double(v)) => Some(v),
            _ => None
        }
    }

    pub fn as_int() -> Option<i64> {
        match self {
            Self::Primitive(Primitive::Integer(v)) => Some(v),
            _ => None
        }
    }
}
