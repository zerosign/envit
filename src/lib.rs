#![feature(is_sorted)]

//!`serde-env` is a library for deserializing environment variables like structure into typesafe structs.
//!
//! # Overview
//!
//! Entrypoint of deserialization for this crates are function `serde_env::deserialize_envs`. It accepts
//! `impl Iterator<Item=Into<Pair<'_>>>`, this iterator can be build from `serde_env::dotenv` function.
//!
//! ```rust
//! use serde_env::{deserialize_envs, dotenv};
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
//!

pub mod de;
pub mod error;
pub mod options;
pub mod types;
pub mod value;

use std::io;

use error::Error;
use std::collections::HashMap;
use types::{Pair, PairSeq};
use value::Value;

#[derive(Debug)]
pub(crate) struct State {
    pub last: Vec<String>,
    pub inner: Value,
}

#[derive(Debug)]
pub(crate) struct Diff {
    pub shared: Vec<String>,
    pub first: Vec<String>,
    pub second: Vec<String>,
}

/// returns similar paths & both diverging path
fn lookup_branch(last: Vec<String>, current: Vec<String>) -> Option<Diff> {
    None
}

///
/// This function will transform recursively [`PairSeq`](PairSeq) into
/// [`Value`](Value).
///
/// I can't use fold since the algorithm are exactly doing linear transform,
/// since when there is branched node in [`PairSeq`](PairSeq), I need to
/// do recursively transform the next iterator by abandoning current iterator flow.
///
fn transform<I>(iter: &mut I, state: Option<State>) -> Result<Value, Error> where I: Iterator<Item = Pair> {

    let r = if let Some(Pair { fields, value }) = iter.next() {
        // fetch current field
        let field = &fields[fields.len() - 1];
        // fetch current parent
        let parent = &fields[0..fields.len() - 1];

        match state {
            // initial state
            None => {
                // create temporal root node
                let mut inner = HashMap::new();

                // TODO(@zerosign) : will use Value::try_from when everything works
                inner.insert(String::from(field), Value::String(item.value.clone()));

                transform(iter, Some(State {
                    last: parent.to_vec(),
                    inner: Value::Object(inner),
                }))
            },
            Some(State { last, inner }) => {
                // check whether last parent is the same as current parent
                if last == parent {
                    // if last parent equal to current parent then insert current field into this inner
                    // means it has the same parent
                    inner.insert(String::from(field), Value::String(value.clone()));
                    transform(iter, Some(State { parent, inner }))
                } else {
                    // if last parent not equal to current parent, then it means
                    // there is branched node somewhere in its parent
                    // and I need to lookup where the path are being branched
                    //
                    // find similarity of both branch
                    //
                    // - if there is similarity, create parent (missing) branch [`Value::Object`](Value::Object), then
                    //   merge two hashmap (last inner and current inner) into new HashMap. While current inner Value are
                    //   recursively created by using another transform calls
                    //
                    // - if there is no parent that matches both branches, just create new hashmap that points into both branch
                    //
                    match lookup_branch(last, parent) {
                        None => {
                            // no parent that matches both branches
                            // create new hashmap
                            let mut root = HashMap::new();

                            // create last branch by iterating branches
                            let mut cursor = root;

                            for n in last.iter() {
                                let mut child = HashMap::new();
                                cursor.insert(n, Value::Object(child));
                                cursor = child;
                            }
                            // insert last branch inner into the last segment
                            cursor.insert(inner);

                            cursor = root;

                            let mut branch = HashMap::new();

                            // transform the rest
                            transform(iter, State {
                                last: parent.to_vec(),
                                inner: Value::Object(branch),
                            }).map(move|v| {
                                // insert the result of the current branch transform
                                // into current root and return current root
                                root.insert(field, v);
                                Value::Object(root)
                            })
                        },
                        Some(Diff { shared, first, second }) => {
                            // TODO(@zerosign): some twisting magic needed in here
                        },
                    }
                }

            }
    } else {
        // this is the base case
        match state {
            Some( State { last, inner } ) => {
                // check last parent is empty or not (mostly it's not) :))
                if last.is_empty() {
                    Ok(inner)
                } else {
                    let parent = &last[last.len()-1];
                    let fields = &last[0..last.len() - 1];

                    let root = fields.iter().fold(Value::Object(HashMap::new()), |&s, item| {
                        match s {
                            Value::Object(&mut h) => {
                                let mut child = HashMap::new();
                                h.insert(item, Value::Object(child)).ok_or(|_| Error::UnknownError(String::from("already exists")))
                            },
                            _ => Error::UnknownError(String::from("type should be an `Value::Object`"))
                        }
                    })?;

                    root.insert(parent, inner).ok_or(|_| Error::UnknownError(String::from("can't insert current inner")))
                }
            },
            _ => Err(Error::UnknownError(String::from("state is empty")))
        }
    };

    Err(Error::UnknownError(String::from("unimplemented")))
}

///
///
///
///
pub fn dotenv<'a, R>(reader: R) -> Result<Value, Error>
where
    R: io::BufRead,
{
    transform(PairSeq::from(reader.lines().filter_map(move |r| match r {
        Ok(ref line) => {
            let line = line.trim();

            // ignore comments
            if !line.starts_with('#') {
                let words = line.split('=').map(|v| v.trim()).collect::<Vec<&str>>();

                match &words[..] {
                    &[key, value] => Some(Pair::new(key.split("__"), value)),
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    })))
}
