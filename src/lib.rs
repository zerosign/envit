//!`serde-env` is a library for deserializing environment variables like structure into typesafe structs.
//!
//! # Overview
//!
//! Entrypoint of deserialization for this crates are function `envit::deserialize_envs`. It accepts
//! `impl Iterator<Item=Into<Pair<'_>>>`, this iterator can be build from `envit::dotenv` function.
//!
//! ```rust
//! use envit::{deserialize_envs, dotenv};
//! use std::io::Cursor;
//!
//! let raw = r#"CONFIG__DATABASE__NAME=name
//! CONFIG__DATABASE__USERNAME=username
//! CONFIG__DATABASE__CREDENTIAL__TYPE=password
//! CONFIG__DATABASE__CREDENTIAL__PASSWORD=some_password
//! CONFIG__DATABASE__CONNECTION__POOL=10
//! CONFIG__DATABASE__CONNECTION__TIMEOUT=10
//! CONFIG__DATABASE__CONNECTION__RETRIES=10,20,30
//! # CONFIG__APPLICATION__ENV=development
//! CONFIG__APPLICATION__LOGGER__LEVEL=info"#;
//!
//! let envs = dotenv(Cursor::new(raw));
//!
//! assert!(deserialize_envs(envs).is_ok());
//!
//! ```
//!
//! The flow :
//!
//! - parse per-line from [`io::Read`](io::Read),
//! - sort iterator based on the keys (array of String) and returns [`PairSeq`](types:PairSeq),
//!   the order of the keys ~ the more it has length + natural order (descending)
//! - construct [`Value::Object`](Value::Object) bottom-up by iterating [`PairSeq`](types::PairSeq).
//!
//!   notes:
//!
//!   - base case : when there is no next in iterator or then parse the rest of the parents, check whether
//!                 there is diverging root and merge it together in one parent
//!
//!   - branch case : create the parent, add current node into parent, then add new leaf by recursively calls
//!                   the function.
//!
//!

#[cfg(feature = "envit_serde")]
pub mod de;
pub mod error;
pub mod options;
#[cfg(feature = "envit_querable")]
pub mod querable;
pub mod types;
pub mod value;

use error::Error;
use std::{borrow::Cow, char, hash::Hash, io};
use types::{Pair, PairSeq};
use value::Value;

///
/// State to holds inner state when doing transform.
///
#[derive(Debug)]
struct State<'a, 'b>
where
    'b: 'a,
{
    pub last: &'a [&'b str],
    pub inner: Value,
}

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
/// This function will transform recursively [`PairSeq`](PairSeq) into
/// [`Value`](Value).
///
/// I can't use fold since the algorithm are exactly doing linear transform,
/// since when there is branched node in [`PairSeq`](PairSeq), I need to
/// do recursively transform the next iterator by abandoning current iterator flow.
///
fn transform<'a, 'b, I>(iter: &mut I, state: Option<State<'a, 'b>>) -> Result<Value, Error>
where
    I: Iterator<Item = Pair<'a>>,
    'a: 'b,
{
    Err(Error::CustomError(String::from("test")))
}

///
///
///
///
pub fn dotenv<'a, R>(reader: R) -> Result<Value, Error>
where
    R: io::BufRead,
{
    transform(
        &mut PairSeq::from(reader.lines().filter_map(move |r| match r {
            Ok(line) => {
                if !line.starts_with('#') {
                    let words = line
                        .split('=')
                        .map(|v| Cow::from(v.trim()))
                        .collect::<Vec<Cow<'_, str>>>();

                    // key & value exists
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
            }
            _ => None,
        }))
        .into_iter(),
        None,
    )
}

#[cfg(test)]
mod test {

    use super::similarity;

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
}
