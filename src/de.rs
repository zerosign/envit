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

use crate::options::{Options, OptionsBuilder};

pub struct Deserializer<'a> {
    options: Options<'a>,
}
