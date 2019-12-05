use std::{
    borrow::Cow,
    collections::{binary_heap, BinaryHeap},
    io,
    iter::FromIterator,
};

use crate::{
    error::{Error, LiteralError},
    serde::{
        de::{self, Error as SerdeError},
        forward_to_deserialize_any,
    },
    types::{StringDict, TreeCursor},
};

pub struct MapAccess<'a> {
    idx: usize,
    inner: TreeCursor<'a>,
}

impl<'de> de::MapAccess<'de> for MapAccess<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        // self.inner.indices[idx];
        Err(Self::Error::EmptyStr)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        Err(Self::Error::EmptyStr)
    }
}

#[derive(Debug, PartialEq)]
enum Sign {
    Signed,
    Unsigned,
}

#[derive(Debug, PartialEq)]
enum State {
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
                Some(State::Int(s)) => {
                    if char::is_digit(ch, 10) {
                        Some(State::Int(s))
                    } else if ch == '.' {
                        Some(State::Double)
                    } else {
                        Some(State::String)
                    }
                }
                // sdouble -> sdouble | string
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
                        Some(State::Int(Sign::Unsigned))
                    } else if ch == '-' {
                        Some(State::Int(Sign::Signed))
                    } else {
                        Some(State::String)
                    }
                }
            }
        }) {
            Some(State::Int(Sign::Unsigned)) => self.deserialize_u64(visitor),
            Some(State::Int(Sign::Signed)) => self.deserialize_i64(visitor),
            Some(State::Double) => self.deserialize_f64(visitor),
            Some(State::String) => self.deserialize_str(visitor),
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
