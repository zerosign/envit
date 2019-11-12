use crate::{
    error::{ArrayError, LiteralError, ValueError},
    types::Parser,
};
use std::{
    borrow::{Borrow, Cow},
    cell::Cell,
    char,
    collections::HashMap,
};

///
/// Number sum types.
///
#[derive(Debug, PartialEq)]
pub enum Number {
    Integer(i64),
    Double(f64),
}

///
/// Literal value sum types.
///
/// We differentiate primitive values from `Value`, so that
/// our array types are flat.
///
#[derive(Debug, PartialEq)]
pub enum Literal {
    Number(Number),
    String(String),
    Bool(bool),
}

impl Literal {
    pub fn integer<V>(v: V) -> Literal
    where
        V: Into<i64>,
    {
        Literal::Number(Number::Integer(v.into()))
    }

    #[inline]
    pub fn double<V>(v: V) -> Literal
    where
        V: Into<f64>,
    {
        Literal::Number(Number::Double(v.into()))
    }

    #[inline]
    pub fn string<S>(s: S) -> Literal
    where
        S: Into<String>,
    {
        Literal::String(s.into())
    }

    #[inline]
    pub fn bool<V>(v: V) -> Literal
    where
        V: Into<bool>,
    {
        Literal::Bool(v.into())
    }
}

///
/// State to represents parsing when scan phase is started.
///
#[derive(Debug, PartialEq)]
enum State {
    String,
    Int,
    Double,
}

impl Parser for Literal {
    type Item = Self;
    type Error = LiteralError;

    /// Parse raw &str into Literal value.
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
    fn parse(raw: &str) -> Result<Self::Item, Self::Error> {
        let raw = raw.trim();

        // empty string or quoted string or boolean
        if raw.is_empty() {
            Err(Self::Error::EmptyStr)
        } else if raw.starts_with('"') && raw.ends_with('"') {
            Ok(Literal::String(String::from(&raw[1..raw.len() - 1])))
        } else if raw == "true" {
            Ok(Literal::Bool(true))
        } else if raw == "false" {
            Ok(Literal::Bool(false))
        } else {
            // need scan phase (diff integer, double and string)
            // raw -> int | double | string .
            // int -> double | string .
            // double -> double | string .
            // string -> string .
            match raw.chars().fold(None, |state, ch| {
                match state {
                    // int -> double | string
                    Some(State::Int) => {
                        if char::is_digit(ch, 10) {
                            Some(State::Int)
                        } else if ch == '.' {
                            Some(State::Double)
                        } else {
                            Some(State::String)
                        }
                    }
                    // double -> double | string
                    Some(State::Double) => {
                        if char::is_digit(ch, 10) {
                            Some(State::Double)
                        } else {
                            Some(State::String)
                        }
                    }
                    // string -> string | bool .
                    Some(State::String) => Some(State::String),
                    // None
                    _ => {
                        if char::is_digit(ch, 10) {
                            Some(State::Int)
                        } else {
                            Some(State::String)
                        }
                    }
                }
            }) {
                Some(State::Int) => raw
                    .parse::<i64>()
                    .map_err(|_| Self::Error::NumberError)
                    .map(|v| Literal::Number(Number::Integer(v))),
                Some(State::Double) => raw
                    .parse::<f64>()
                    .map_err(|_| Self::Error::NumberError)
                    .map(|v| Literal::Number(Number::Double(v))),
                Some(State::String) => Ok(Literal::String(String::from(raw))),
                _ => Err(Self::Error::EmptyStr),
            }
        }
    }
}

///
/// Array inner types.
///
/// We use FlatArray types since in envs we could only
/// model this kind of array.
///
#[derive(Debug, PartialEq)]
pub struct FlatArray(Vec<Literal>);

impl Default for FlatArray {
    #[inline]
    fn default() -> Self {
        Self(Vec::with_capacity(0))
    }
}

///
/// recursively find index where separator `sep` is located for quoted str.
///
pub(crate) fn lookup_quoted_sep<'a, S>(raw: S, sep: char) -> Option<usize>
where
    S: Into<Cow<'a, str>>,
{
    let raw = raw.into();
    // found quoted str
    match raw.find(sep) {
        Some(idx) => {
            if &raw[idx - 1..idx] == "\\" {
                // escaped str, skip
                lookup_quoted_sep(&raw[idx + 1..raw.len()], sep).map(|v| v + idx)
            } else {
                Some(idx)
            }
        }
        _ => None,
    }
}

///
/// recursively find index where array separator `sep` is located in array
/// that may contains quoted str.
///
pub(crate) fn lookup_array_sep<'a, S>(raw: S, sep: char) -> Option<usize>
where
    S: Into<Cow<'a, str>>,
{
    let raw = raw.into();
    if raw.starts_with('"') {
        match lookup_quoted_sep(&raw[1..raw.len()], '"') {
            Some(idx) => lookup_quoted_sep(&raw[idx..raw.len()], sep).map(|v| idx + v),
            _ => None,
        }
    } else {
        raw.find(sep)
    }
}

impl Parser for FlatArray {
    type Item = Self;
    type Error = ArrayError;

    ///
    /// parser implementation for parsing FlatArray.
    ///
    /// flat array :
    /// flat_array -> '[' primitive ( ',' primitive )* ']'
    ///
    /// things to note :
    ///
    /// - we need to be able to escape quotation in quoted string when trying to find
    ///  ','.
    ///
    fn parse(raw: &str) -> Result<Self::Item, Self::Error> {
        let raw = raw.trim();

        if raw.is_empty() {
            Err(Self::Error::EmptyStr)
        } else if raw.starts_with('[') && raw.ends_with(']') {
            // detect array value
            // if its an array split based on ',' (careful of quoted str)
            let slices = Cow::from(raw[1..raw.len() - 1].trim());

            if slices.is_empty() {
                Ok(Self::default())
            } else {
                let mut data = vec![];
                let mut cursor = slices;
                //
                // quote escape by using lookup separator function.
                // the idea is actively looking for separator but also
                // do escape quotation by skipping quoted str.
                //
                // when idx where separator found, slice the str in [idx + 1]
                //
                loop {
                    let part_idx = lookup_array_sep(cursor.clone(), ',').unwrap_or(cursor.len());
                    let current = &cursor[0..part_idx];

                    let el = Literal::parse(current).map_err(Self::Error::LiteralError)?;
                    data.push(el);

                    // TODO @zerosign : there should be better way to do this
                    // this need to re assign cursor but cursor already being borrowed in `Literal::parse`
                    if part_idx < cursor.len() - 1 {
                        cursor = String::from(&cursor[part_idx + 1..cursor.len()]).into();
                    } else {
                        break;
                    }
                }

                Ok(Self(data))
            }
        } else {
            Err(Self::Error::ParseError)
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
    Literal(Literal),
    // in here array only can holds [`Literal`](Literal) values.
    // no array in array or object in array allowed.
    Array(FlatArray),
    Object(HashMap<String, Value>),
}

impl Value {
    #[inline]
    pub fn integer<V>(v: V) -> Value
    where
        V: Into<i64>,
    {
        Value::Literal(Literal::integer(v))
    }

    #[inline]
    pub fn double<V>(v: V) -> Value
    where
        V: Into<f64>,
    {
        Value::Literal(Literal::double(v))
    }

    #[inline]
    pub fn string<S>(s: S) -> Value
    where
        S: Into<String>,
    {
        Value::Literal(Literal::string(s))
    }

    #[inline]
    pub fn empty_list() -> Value {
        Value::Array(FlatArray(vec![]))
    }

    #[inline]
    pub fn bool<V>(v: V) -> Value
    where
        V: Into<bool>,
    {
        Value::Literal(Literal::bool(v))
    }

    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Literal(Literal::String(ref s)) => Some(s),
            _ => None,
        }
    }

    #[inline]
    pub fn insert(&mut self, k: String, v: Self) -> Option<Value> {
        match self {
            Self::Object(inner) => inner.insert(k, v),
            _ => None,
        }
    }

    #[inline]
    pub fn as_vec(&self) -> Option<&Vec<Literal>> {
        match self {
            Self::Array(FlatArray(s)) => Some(s),
            _ => None,
        }
    }

    #[inline]
    pub fn as_double(&self) -> Option<f64> {
        match self {
            Self::Literal(Literal::Number(Number::Double(v))) => Some(*v),
            _ => None,
        }
    }

    #[inline]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Literal(Literal::Number(Number::Integer(v))) => Some(*v),
            _ => None,
        }
    }
}

impl Parser for Value {
    type Item = Value;
    type Error = ValueError;

    ///
    /// WARN: you shouldn't use this method to parse object since
    ///       it won't give the output to an Value::Object
    ///
    /// TODO(@zerosign) : eliminate double checks in
    ///                   parsing both `Literal` & `FlatArray`
    ///
    #[inline]
    fn parse(raw: &str) -> Result<Self::Item, Self::Error> {
        let raw = raw.trim();

        if raw.is_empty() {
            Err(Self::Error::EmptyStr)
        } else if raw.starts_with('[') && raw.starts_with(']') {
            FlatArray::parse(raw)
                .map(Value::Array)
                .map_err(ValueError::ArrayError)
        } else {
            Literal::parse(raw)
                .map(Value::Literal)
                .map_err(ValueError::LiteralError)
        }
    }
}

macro_rules! literal_conv {
    ($($conv:path => [$($src:ty),*]),*) => {
        $($(impl From<$src> for Literal {

            #[inline]
            fn from(v: $src) -> Self {
                $conv(v)
            }
        })*)*
    }
}

//
// Note: we don't support u64 at this point.
//
literal_conv!(
    Literal::integer => [u8, u16, u32, i8, i16, i32, i64],
    Literal::double  => [f32, f64],
    Literal::string  => [String, &'static str],
    Literal::bool    => [bool]
);

macro_rules! array {
    [] => (FlatArray::default());
    [$($val:expr),*] => (FlatArray(<[_]>::into_vec(Box::new([$(Literal::from($val)),*]))));
}

#[cfg(test)]
mod test {

    use crate::{
        error::{ArrayError, LiteralError, ValueError},
        types::Parser,
        value::{FlatArray, Literal},
    };

    #[test]
    fn test_simple_literal_parsing() {
        let samples = vec!["", r#""""#, "true", "false"];
        let expected = vec![
            Err(LiteralError::EmptyStr),
            Ok(Literal::string("")),
            Ok(Literal::bool(true)),
            Ok(Literal::bool(false)),
        ];

        for (idx, sample) in samples.iter().enumerate() {
            let result = Literal::parse(sample);
            assert_eq!(result, expected[idx]);
        }
    }

    #[test]
    fn test_literal_parsing() {
        let samples = vec![
            "12121212121",
            "23123123.231231",
            "hello world",
            "  test test ",
            r#""
""#,
        ];

        let expected: Vec<Result<_, LiteralError>> = vec![
            Ok(Literal::integer(12121212121i64)),
            Ok(Literal::double(23123123.231231)),
            Ok(Literal::string("hello world")),
            Ok(Literal::string("test test")),
            Ok(Literal::string("\n")),
        ];

        for (idx, sample) in samples.iter().enumerate() {
            let result = Literal::parse(sample);
            assert_eq!(result, expected[idx]);
        }
    }

    #[test]
    fn test_array_parsing() {
        let samples = vec![
            "[]",
            "[1,    2]",
            "[3, true, false,        true]",
            r#"["test", "hello world", 2, 3]"#,
        ];

        let expected: Vec<Result<_, ArrayError>> = vec![
            Ok(FlatArray::default()),
            Ok(array![1, 2]),
            Ok(array![3, true, false, true]),
            Ok(array!["test", "hello world", 2, 3]),
        ];

        for (idx, sample) in samples.iter().enumerate() {
            let result = FlatArray::parse(sample);
            assert_eq!(result, expected[idx]);
        }
    }
}
