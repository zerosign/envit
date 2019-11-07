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

///
/// returns similar paths & both diverging path
///

#[inline]
fn similarity<'a, I>(last: I, current: I) -> impl Iterator<Item = &'a String>
where
    I: IntoIterator<Item = &'a String>,
{
    last.into_iter()
        .zip(current)
        .filter_map(|(l, r)| if String::eq(l, r) { Some(l) } else { None })
}

///
/// create dummy childs for path iterator `iter` on `root`.
///
#[inline]
fn dummy_childs<'a, I>(iter: &I, root: &HashMap<String, Value>) -> HashMap<String, Value>
where
    I: Iterator<Item = &'a String>,
{
    iter.fold(*root, move |mut s, item| {
        let child = HashMap::new();
        s.insert(String::from(item), Value::Object(child));
        s
    })
}

///
/// This function will transform recursively [`PairSeq`](PairSeq) into
/// [`Value`](Value).
///
/// I can't use fold since the algorithm are exactly doing linear transform,
/// since when there is branched node in [`PairSeq`](PairSeq), I need to
/// do recursively transform the next iterator by abandoning current iterator flow.
///
fn transform<I>(iter: &mut I, state: Option<State>) -> Result<Value, Error>
where
    I: Iterator<Item = Pair>,
{
    match iter.next() {
        Some(Pair { fields, value }) => {
            // fetch current field
            let field = &fields[fields.len() - 1];
            // fetch current parent
            let parent = &fields[0..fields.len() - 1];

            match state {
                // initial state
                None => {
                    // create temporal root node
                    let mut inner = HashMap::new();

                    // TODO(@zerosign) : will use `Value::parse` when everything works
                    inner.insert(String::from(field), Value::string(value));

                    transform(
                        iter,
                        Some(State {
                            last: parent.to_vec(),
                            inner: Value::Object(inner),
                        }),
                    )
                }
                Some(State {
                    last: mut last,
                    inner: Value::Object(mut inner),
                }) => {
                    // check whether last parent is the same as current parent
                    if last == parent {
                        // if last parent equal to current parent then insert current field into this inner
                        // means it has the same parent
                        // TODO(@zerosign) : will use `Value::parse` when everything works
                        inner.insert(String::from(field), Value::string(value));
                        transform(
                            iter,
                            Some(State {
                                last: parent.to_vec(),
                                inner: Value::Object(inner),
                            }),
                        )
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
                        // A__B__C
                        // B__C
                        // last : A__B, C
                        //
                        let mut parent = parent.to_vec();
                        let mut similar: Vec<String> = similarity(last.iter(), parent.iter())
                            .map(String::from)
                            .collect();

                        if similar.is_empty() {
                            // no parent that matches both branches
                            // create new hashmap
                            let mut root = HashMap::new();

                            // create last branch by iterating branches
                            // since last is parent and its actually point up to
                            // current path
                            let mut cursor =
                                dummy_childs(&last[0..last.len() - 1].iter(), &mut root);

                            // insert last branch inner into the last segment
                            // field of inner ~ last field in last
                            cursor.insert(last[last.len() - 1], Value::Object(inner));

                            // create new leaf branch that holds current field & value
                            let mut next = HashMap::new();

                            // TODO(@zerosign) : will use `Value::parse` when everything works
                            next.insert(*field, Value::string(value));

                            // transform the rest
                            transform(
                                iter,
                                Some(State {
                                    last: parent,
                                    inner: Value::Object(next),
                                }),
                            )
                            .map(move |v| {
                                // insert the result of the current branch transform
                                // into current root and return current root
                                root.insert(fields[0], v);
                                Value::Object(root)
                            })
                        } else {
                            let idx = similar.len() - 1;

                            // first
                            let _ = &last[idx..last.len()];

                            let mut root = HashMap::new();

                            // cursor
                            let _ = dummy_childs(&similar.iter(), &root);

                            // TODO(@zerosign): some twisting magic needed in here

                            // transform the rest
                            // transform(iter, State {
                            //     last: parent.to_vec(),
                            //     inner: Value::Object(next),
                            // }).map(move|v| {
                            //     // insert the result of the current branch transform
                            //     // into current root and return current root
                            //     root.insert(&fields[0], v);
                            //     Value::Object(root)
                            // })
                            Err(Error::CustomError(String::from("unimplemented")))
                        }
                    }
                }
            }
        }
        _ => {
            // this is the base case
            match state {
                Some(State { last, inner }) => {
                    // check last parent is empty or not (mostly it's not) :))
                    if last.is_empty() {
                        Ok(inner)
                    } else {
                        let parent = &last[last.len() - 1];
                        let fields = &last[0..last.len() - 1];
                        let mut root = HashMap::new();
                        let _ = dummy_childs(&fields.iter(), &mut root);
                        root.insert(String::from(parent), inner);
                        Ok(Value::Object(root))
                    }
                }
                _ => {
                    //
                    Err(Error::CustomError(String::from("state is empty")))
                }
            }
        }
    }
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
            Ok(ref line) => {
                let line = line.trim();

                // ignore comments
                if !line.starts_with('#') {
                    let words = line.split('=').map(|v| v.trim()).collect::<Vec<&str>>();

                    match &words[..] {
                        [key, value] => Some(Pair::new(key.split("__"), value)),
                        _ => None,
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
