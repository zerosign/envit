#![feature(pattern)]

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

pub mod de;
pub mod error;
pub mod options;
#[cfg(feature = "envit_querable")]
pub mod querable;
// pub mod ser;
pub mod types;
pub mod value;

// use error::Error;
// use std::{borrow::Cow, io};
// use types::{Pair, PairSeq};
// use value::Value;
