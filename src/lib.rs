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
//!

pub mod de;
pub mod error;
pub mod options;
pub mod types;
pub mod value;

use std::io::BufRead;

use error::{Error, PairError};
use types::{Pair, PairSeq};

//
//
//
//
pub fn dotenv<'a, R>(reader: R) -> PairSeq<'a>
where
    R: io::BufRead,
{
    PairSeq::from(reader.lines().filter_map(move |r| match r {
        Ok(ref line) => {
            let line = line.trim();

            // ignore comments
            if !line.starts_with('#') {
                let words = line.split('=').map(|v| v.trim()).collect::<Vec<&str>>();

                match &words[..] {
                    &[key, value] => Some(Pair::new(String::from(key), String::from(value))),
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    }))
}

// Check whether there is anomaly of duplicated inferred value
// in the pairs, we assume that the input is already sorted.
//
// Example:
//
// ```env
// DATABASE_USER_NAME="test"
// DATABASE_USER="test"
// ```
//
// With this there is a conflict between `database.user` types, since
// it can be String or Object.
//
pub fn verify_tree<'a, I>(iter: I) -> Result<I, Error>
where
    I: Iterator<Item = Pair<'a>>,
{
    // check whether iter is sorted or not
    // if iter is not sorted then return the error
    if iter.is_sorted_by_key(|pair| pair.field()) {
        Ok(iter)
    } else {
        Err(Error::UnsortedError)
    }
}
