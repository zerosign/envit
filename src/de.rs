//! # Deserialization inner workings
//!
//! To deserializing with serde, we can't directly serialize it with Deserializer (using str), since
//! it will incurs a lot of overheads when querying siblings or parent when traversing in the streams.
//! We need to have a complete picture of the tree first, since serde require us to have an in-place
//! update for creating the value.
//!
//! Current idea :
//!
//! - deserialize the raw types (`&str` or `Reader`) into same `PairSeq`
//! - construct `Value::Object` bottom up by iterating sorted `PairSeq` iterator.
//!
//! In deserializer, the only valid entrypoint (from outside/callee) are
//! (Deserializer::deserialize_any, Deserializer::deserialize_map, Deserializer::deserialize_struct),
//! Other than that it's invalid since there were no envs that has only value, it need to
//! be in pairs (key & value).
//!
//!
use crate::{
    error::Error,
    types::{Pair, PairSeq, Parser},
    value::Value,
};
use serde::{
    de::{self, Error as SerdeError},
    forward_to_deserialize_any,
};
use std::{
    borrow::Cow,
    collections::{binary_heap::IntoIter, HashMap},
    io::{self, Cursor},
};

///
/// returns similar paths
///
fn subset<I, T>(lhs: I, rhs: I) -> impl Iterator<Item = T>
where
    I: IntoIterator<Item = T>,
    T: Eq,
{
    lhs.into_iter()
        .zip(rhs.into_iter())
        .filter_map(|(l, r)| if l.eq(&r) { Some(l) } else { None })
}

///
/// create dummy childs for path iterator `iter` on `root`.
///
#[inline]
fn dummy_childs<I>(
    iter: I,
    root: &mut HashMap<String, Value>,
) -> Result<HashMap<String, Value>, Error>
where
    I: IntoIterator<Item = String>,
{
    // TODO @zerosign maybe just use ValueMut from HashMap ?
    Ok(iter.into_iter().fold(root, move |s, item| {
        let mut child = HashMap::new();
        s.insert(item, Value::Object(child));
        s
    }))
}

///
/// State to holds inner state when doing transform.
///
#[derive(Debug)]
struct State<'a> {
    pub last: Cow<'a, [String]>,
    pub inner: Value,
}

pub struct Deserializer<'a, 'b>
where
    'b: 'a,
{
    inner: IntoIter<Pair<'b>>,
    state: Option<State<'a>>,
}

impl<'a, 'b> Deserializer<'a, 'b>
where
    'b: 'a,
{
    ///
    /// This will calls `Deserializer::from_reader` underneath.
    ///
    #[inline]
    pub fn from_str(raw: &str) -> Self {
        Self::from_reader(Cursor::new(raw))
    }

    ///
    /// entry point that fetchs from `io::BufRead`.
    ///
    /// In here we will sort lines based (max heap) based on fields path.
    ///
    pub fn from_reader<R>(reader: R) -> Self
    where
        R: io::BufRead,
    {
        let iter = PairSeq::from(reader.lines().filter_map(move |r| {
            r.ok().and_then(move |line| {
                if !line.starts_with('#') {
                    let words = line
                        .split('=')
                        .map(|v| Cow::from(v.trim()))
                        .collect::<Vec<Cow<'_, str>>>();

                    if words.len() == 2 {
                        let key = words[0].clone();

                        Some(Pair::new(
                            key.split("__").map(String::from),
                            String::from(words[1].clone()),
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        }))
        .into_iter();

        Self {
            inner: iter,
            state: None,
        }
    }
}

impl<'de, 'a, 'b> de::Deserializer<'de> for &'a mut Deserializer<'de, 'b> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if let Some(Pair { fields, value }) = self.inner.next() {
            // fetch current field A -> B, where B ~ value non Value::Object
            let current_field = &fields[fields.len() - 1];
            // fetch current parent of the field where there is C -> A, where A -> B,
            // B ~ value non Value::Object and thus C is the parent
            let current_parent = fields[0..fields.len() - 1]
                .iter()
                .map(String::from)
                .collect::<Vec<_>>();

            match &mut self.state {
                // initial state
                None => {
                    let mut inner = HashMap::new();
                    let value = Value::parse(&value).map_err(Self::Error::ParseError)?;

                    inner.insert(String::from(current_field), value);

                    self.state = Some(State {
                        last: Cow::from(current_parent),
                        inner: Value::Object(inner),
                    });

                    // we should visit the inner when a node is done recreated (including it's children)

                    self.deserialize_any(visitor)
                }
                Some(State {
                    last: last,
                    inner: inner,
                }) => {
                    match inner {
                        Value::Object(_) => {
                            // check whether last parent is the same as current parent
                            if *last == current_parent {
                                // if last parent equal to current parent then insert current field into this inner
                                let value =
                                    Value::parse(&value).map_err(Self::Error::ParseError)?;
                                // this assume that the object are Value::Object
                                inner.insert(String::from(current_field), value);

                                self.deserialize_any(visitor)
                            } else {
                                // if last parent not equal to current parent, then it means
                                // there is branched node somewhere in its parent
                                // and I need to lookup where the path are being branched
                                //
                                // find similarity of both branch
                                //
                                // - if there is similarity, create parent (missing) branch
                                //   [`Value::Object`](Value::Object), then
                                //   merge two hashmap (last inner and current inner) into
                                //   new HashMap. While current inner Value are
                                //   recursively created by using another transform calls
                                //
                                // - if there is no parent that matches both branches, just
                                //   create new hashmap that points into both branch
                                let similars = subset(Vec::from(last.clone()), current_parent)
                                    .collect::<Vec<String>>();

                                if similars.is_empty() {
                                    // no parent that matches both branches
                                    // create new hashmap
                                    let mut root = HashMap::new();

                                    let mut cursor = dummy_childs(
                                        Vec::from(&last[0..last.len() - 1]),
                                        &mut root,
                                    )?;
                                }

                                Err(Self::Error::custom("unimplemented"))
                            }
                        }
                        _ => Err(Self::Error::custom("type error, should be `Value::Object`")),
                    }
                }
                _ => Err(Self::Error::custom("unimplemented")),
            }
        } else {
            Err(Self::Error::custom("eof"))
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::custom("unimplemented"))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::custom("unsupported"))
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::custom("unimplemented"))
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::custom("unimplemented"))
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::custom("unimplemented"))
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::custom("unimplemented"))
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::custom("unimplemented"))
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::custom("unimplemented"))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::custom("unimplemented"))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::custom("unimplemented"))
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::custom("unimplemented"))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::custom("unimplemented"))
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::custom("unimplemented"))
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::custom("unimplemented"))
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::custom("unimplemented"))
    }
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::custom("unimplemented"))
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::custom("unimplemented"))
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::custom("unimplemented"))
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::custom("unimplemented"))
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::custom("unimplemented"))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::custom("unimplemented"))
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::custom("unimplemented"))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Self::Error::custom("unimplemented"))
    }

    forward_to_deserialize_any! {
        struct enum identifier ignored_any tuple
    }
}

///
/// `serde` convention need toplevel module `from_str`
/// that accepts `&str`.
///
///
pub fn from_str<'a, T>(s: &'a str) -> Result<T, Error>
where
    T: de::Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str(s);

    // TODO: @zerosign should we check whitespace in here ?
    Ok(T::deserialize(&mut deserializer)?)
}

#[cfg(test)]
mod test {
    use super::{subset, Deserializer};
    use std::borrow::Cow;

    static ENV_SAMPLE: &'static str = r#"CONFIG__DATABASE__NAME=name
CONFIG__DATABASE__USERNAME=username
CONFIG__DATABASE__CREDENTIAL__TYPE=password
CONFIG__DATABASE__CREDENTIAL__PASSWORD=some_password
CONFIG__DATABASE__CONNECTION__POOL=10
CONFIG__DATABASE__CONNECTION__TIMEOUT=10
CONFIG__DATABASE__CONNECTION__RETRIES=10,20,30
# CONFIG__APPLICATION__ENV=development
CONFIG__APPLICATION__LOGGER__LEVEL=info"#;

    #[test]
    fn test_check_similarity() {
        let left = Cow::from(vec![String::from("test"), String::from("test")]);
        let right = Cow::from(vec![String::from("test")]);

        assert!(!subset(left, right).is_empty());
    }

    #[test]
    fn test_init_deserializer_from_str() {
        Deserializer::from_str(ENV_SAMPLE);
    }
}
