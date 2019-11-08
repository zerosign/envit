//! # Deserialization inner workings
//!
//! To deserializing with serde, we can't directly serialize it with Deserializer (using str), since
//! it will incurs a lot of overheads when querying siblings or parent when traversing in the streams.
//! We need to have a complete picture of the tree first, since serde require us to have an in-place
//! update for creating the value.
//!
//! Current idea :
//!
//! - deserialize the raw types (`Vars` or `Reader`) into same `PairSeq`
//! - construct actual mutable tree by traversing `PairSeq` from rows then to cols
//! - when traversing we create intermediate indexes (PairIndex & Relations)
//! - after that, we traverse `Relations` back in serde::de::Deserializer functions.
//!
//!
//! In deserializer, the only valid entrypoint (from outside/callee) are
//! (Deserializer::deserialize_any, Deserializer::deserialize_map, Deserializer::deserialize_struct),
//! Other than that it's invalid since there were no envs that has only value, it need to
//! be in pairs (key & value).
//!
//!
use crate::{
    types::{Pair, PairSeq},
    value::Value,
};
use serde::de::Deserialize;
use std::{
    borrow::Cow,
    collections::binary_heap::IntoIter,
    io::{self, Cursor},
    str::pattern::Pattern,
};

///
/// returns similar paths & both diverging path
///
#[inline]
fn similarity<'a, I, S>(last: I, current: I) -> impl Iterator<Item = S>
where
    I: IntoIterator<Item = S>,
    S: Into<Cow<'a, str>> + Eq,
{
    last.into_iter()
        .zip(current)
        .filter_map(|(l, r)| if l.eq(&r) { Some(l) } else { None })
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

#[cfg(test)]
mod test {

    use super::{similarity, Deserializer};

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
        assert_eq!(
            similarity(["test", "test"].iter().cloned(), ["test"].iter().cloned())
                .collect::<Vec<&str>>(),
            vec!["test"],
        );

        assert!(
            similarity([" test", "test"].iter().cloned(), ["test"].iter().cloned())
                .collect::<Vec<&str>>()
                .is_empty(),
        );
    }

    #[test]
    fn test_init_deserializer_from_str() {
        Deserializer::from_str(ENV_SAMPLE);
    }
}
