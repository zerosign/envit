use std::{
    borrow::Cow,
    collections::{binary_heap, hash_map::DefaultHasher, BinaryHeap, HashSet},
    default::Default,
    hash::{Hash, Hasher},
    io,
    iter::FromIterator,
};

use crate::{
    error::{Error, LiteralError},
    serde::{
        de::{self, Error as SerdeError},
        forward_to_deserialize_any,
    },
    types::{Cursor, Index, Kind as DictKind, StringDict},
};

pub enum Kind {
    Object,
    Literal,
}

impl From<DictKind> for Kind {
    #[inline]
    fn from(kind: DictKind) -> Self {
        match kind {
            DictKind::Leaf => Kind::Literal,
            DictKind::Node => Kind::Object,
        }
    }
}

enum State {
    Available { inner: HashSet<u64>, idx: Index },
    Done,
}

impl Default for State {
    #[inline]
    fn default() -> Self {
        Self::Available {
            inner: HashSet::default(),
            idx: Index::default(),
        }
    }
}

pub struct Deserializer<'a> {
    state: State,
    inner: StringDict<'a>,
}

impl<'a> Deserializer<'a> {
    #[inline]
    pub fn new<'b>(dict: StringDict<'b>) -> Self
    where
        'b: 'a,
    {
        Self {
            state: State::default(),
            inner: dict,
        }
    }

    // None means short circuit
    fn next(&mut self) -> Option<(Index, Kind)> {
        let state = &self.state;
        let dict = &self.inner;

        match state {
            State::Available { inner, idx } => {
                let old_idx = *idx;
                // out of index are being handled directly in method `peek_kind`
                // if there is no index peek_kind will returns None
                // thus this method will actually returns None
                self.peek_kind().and_then(|kind| match kind {
                    Kind::Literal => {
                        let new_idx = idx.next().clone();

                        self.state = if dict.is_available(new_idx) {
                            // hashed already exists

                            State::Available {
                                inner: inner.clone(),
                                idx: new_idx,
                            }
                        } else {
                            State::Done
                        };

                        Some((old_idx, Kind::Literal))
                    }
                    Kind::Object => {
                        // don't need to check since if an object,
                        // there should always be next
                        self.state = State::Available {
                            inner: inner.clone(),
                            idx: idx.down(),
                        };

                        Some((old_idx, Kind::Object))
                    }
                })
            }
            _ => None,
        }
    }

    #[inline]
    fn peek_kind(&self) -> Option<Kind> {
        match &self.state {
            State::Available { inner: _inner, idx } => {
                self.inner.fetch_index_kind(*idx).map(Kind::from)
            }
            _ => None,
        }
    }

    #[inline]
    fn fetch_value(&self, index: Index) -> Option<String> {
        self.inner.fetch_value(index)
    }
}

// impl<'de> de::Deserializer<'de> for Deserializer<'de> {
//     type Error = Error;

//     ///
//     /// only able to deserialize literal value
//     /// other than that give error to deserializer
//     ///
//     fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         Err(Self::Error::EmptyStr)
//     }
// }

#[derive(Debug, PartialEq)]
enum Sign {
    Signed,
    Unsigned,
}

#[derive(Debug, PartialEq)]
enum LiteralState {
    String,
    Int(Sign),
    Double,
}

/// Literal deserializer, (non top-level deserializer)
///
#[derive(Debug, Clone, Copy)]
struct LiteralDeserializer<'de> {
    // already trimmed str
    inner: &'de str,
}

impl<'de> LiteralDeserializer<'de> {
    #[inline]
    pub fn from_str<'b>(s: &'b str) -> Result<Self, LiteralError>
    where
        'b: 'de,
    {
        let s = s.trim();

        if s.is_empty() {
            Err(LiteralError::EmptyStr)
        } else {
            Ok(Self { inner: s })
        }
    }

    fn parse_literal<V>(&self, visitor: V) -> Result<V::Value, LiteralError>
    where
        V: de::Visitor<'de>,
    {
        use serde::Deserializer;

        match self.inner.chars().fold(None, |state, ch| {
            match state {
                // int(s) -> double | string
                Some(LiteralState::Int(s)) => {
                    if char::is_digit(ch, 10) {
                        Some(LiteralState::Int(s))
                    } else if ch == '.' {
                        Some(LiteralState::Double)
                    } else {
                        Some(LiteralState::String)
                    }
                }
                // sdouble -> sdouble | string
                Some(LiteralState::Double) => {
                    if char::is_digit(ch, 10) {
                        Some(LiteralState::Double)
                    } else {
                        Some(LiteralState::String)
                    }
                }
                // string -> string | bool .
                Some(LiteralState::String) => Some(LiteralState::String),
                // None
                _ => {
                    if char::is_digit(ch, 10) {
                        Some(LiteralState::Int(Sign::Unsigned))
                    } else if ch == '-' {
                        Some(LiteralState::Int(Sign::Signed))
                    } else {
                        Some(LiteralState::String)
                    }
                }
            }
        }) {
            Some(LiteralState::Int(Sign::Unsigned)) => self.deserialize_u64(visitor),
            Some(LiteralState::Int(Sign::Signed)) => self.deserialize_i64(visitor),
            Some(LiteralState::Double) => self.deserialize_f64(visitor),
            Some(LiteralState::String) => self.deserialize_str(visitor),
            _ => Err(LiteralError::EmptyStr),
        }
    }
}

// macro_rules! forward_calls {
//     ([($fn:ident),*] => $target:ident) => $({
//         fn $de<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//         where
//             V: de::Visitor<'de> {
//er
//         }
//     })
// }

impl<'de> de::Deserializer<'de> for LiteralDeserializer<'de> {
    type Error = LiteralError;

    ///
    /// only able to deserialize literal value
    /// other than that give error to deserializer
    ///
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.inner.starts_with('"') {
            self.deserialize_str(visitor)
        } else if self.inner == "true" || self.inner == "false" {
            self.deserialize_bool(visitor)
        } else {
            self.parse_literal(visitor)
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.inner.starts_with('"') && self.inner.ends_with('"') {
            visitor.visit_string(self.inner[1..self.inner.len() - 1].to_string())
        } else {
            visitor.visit_string(self.inner.to_string())
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.inner.starts_with('"') && self.inner.ends_with('"') {
            visitor.visit_str(&self.inner[1..self.inner.len() - 1])
        } else {
            visitor.visit_str(&self.inner)
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.inner == "true" {
            visitor.visit_bool(true)
        } else if self.inner == "false" {
            visitor.visit_bool(false)
        } else {
            Err(LiteralError::SyntaxError)
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.inner
            .parse::<u8>()
            .map_err(LiteralError::custom)
            .and_then(move |v| visitor.visit_u8(v))
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.inner
            .parse::<u16>()
            .map_err(Self::Error::custom)
            .and_then(move |v| visitor.visit_u16(v))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.inner
            .parse::<u32>()
            .map_err(Self::Error::custom)
            .and_then(move |v| visitor.visit_u32(v))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.inner
            .parse::<u64>()
            .map_err(Self::Error::custom)
            .and_then(move |v| visitor.visit_u64(v))
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.inner
            .parse::<i8>()
            .map_err(Self::Error::custom)
            .and_then(move |v| visitor.visit_i8(v))
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.inner
            .parse::<i16>()
            .map_err(Self::Error::custom)
            .and_then(move |v| visitor.visit_i16(v))
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.inner
            .parse::<i32>()
            .map_err(Self::Error::custom)
            .and_then(move |v| visitor.visit_i32(v))
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.inner
            .parse::<i64>()
            .map_err(Self::Error::custom)
            .and_then(move |v| visitor.visit_i64(v))
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.inner
            .parse::<f32>()
            .map_err(Self::Error::custom)
            .and_then(move |v| visitor.visit_f32(v))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.inner
            .parse::<f64>()
            .map_err(Self::Error::custom)
            .and_then(move |v| visitor.visit_f64(v))
    }

    forward_to_deserialize_any! {
        char bytes byte_buf option unit unit_struct
        tuple
    }

    fn deserialize_newtype_struct<V>(
        self,
        field: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::Unsupported)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::Unsupported)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::Unsupported)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::Unsupported)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::Unsupported)
    }

    fn deserialize_tuple_struct<V>(
        self,
        field: &'static str,
        idx: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::Unsupported)
    }

    fn deserialize_struct<V>(
        self,
        kind: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::Unsupported)
    }

    fn deserialize_enum<V>(
        self,
        kind: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::Unsupported)
    }
}

#[cfg(test)]
mod test {
    use crate::serde::de::{Deserialize, Deserializer, Visitor};
    use crate::{de::LiteralDeserializer, error::LiteralError};

    #[test]
    fn test_literal_serde() {
        let data = vec![
            "1",
            "\"test hello world\"",
            "true",
            "false",
            "hello world",
            "2.0",
        ];

        data.iter()
            .map(move |s| LiteralDeserializer::from_str(s))
            .for_each(move |de| {
                assert!(de.is_ok());
                let de = de.unwrap();
            });
    }
}
