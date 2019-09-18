//! # Overview
//!
//! Pair should be able to be sorted by key, so that there we don't need
//! to query the node siblings when recurse propagating back into the parent.
//!
//!
//!

use itertools::Itertools;
use std::{borrow::Cow, collections::HashMap};

// Temporary intermediate struct to holds pair of string from envs.
//
// ```rust
//
// let p : Pair = Pair::new("test", "value").into();
//
// assert_eq!(p.field(), "test");
// assert_eq!(p.value(), "value");
//
// ```
//
#[derive(Debug)]
pub struct Pair<'a>(Cow<'a, str>, Cow<'a, str>);

impl<'a> Into<Pair<'a>> for Pair<'a> {
    fn into(self) -> Pair<'a> {
        Pair(self.0, self.1)
    }
}

impl<'a> Pair<'a> {
    #[inline]
    pub fn new<S>(l: S, r: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Pair(l.into(), r.into())
    }

    #[inline]
    pub fn field(&self) -> Cow<'a, str> {
        self.0.clone()
    }

    #[inline]
    pub fn value(&self) -> Cow<'a, str> {
        self.1.clone()
    }
}

//
// Newtype that holds table lookup for parent & children relations.
//
// let relations = Relations::new();
//
//
pub type Relations = HashMap<Option<usize>, Vec<usize>>;

impl Relations {
    #[inline]
    pub fn new() -> Relations {
        Relations(HashMap::new())
    }

    #[inline]
    pub fn lookup(&self, id: usize) -> Option<Vec<usize>> {
        self.get(Some(id))
    }

    #[inline]
    pub fn root(&self) -> Option<Vec<usize>> {
        self.get(None)
    }
}

pub type PathIndex<'a> = Vec<Cow<'a, str>>;

pub type LeafIndex<'a> = HashMap<usize, Cow<'a, str>>;

// #[derive(Debug)]
// pub enum Node<'a, V>
// where
//     V: Sized,
// {
//     Branch {
//         path: NodePath<'a>,
//         children: Vec<Node<'a, V>>,
//     },
//     Leaf {
//         path: NodePath<'a>,
//         value: V,
//     },
// }

// //
// // Pair should be able to be sorted by key, so that
// // when we build the tree there will be no random query
// // to go back to each siblings.
// //
// impl PartialOrd for Pair {
//     fn partial_cmp(&self, other: &Pair) -> Option<Ordering> {
//         Some(self.0.cmp(other.0))
//     }
// }

#[derive(Debug)]
pub struct PairSeq<'a>(Vec<Pair<'a>>);

impl<'a, I, T> From<I> for PairSeq<'a>
where
    T: Into<Pair<'a>>,
    I: Iterator<Item = T>,
{
    #[inline]
    fn from(iter: I) -> Self {
        PairSeq(
            iter.map(move |p| p.into())
                .sorted_by_key(move |p| p.field())
                .collect(),
        )
    }
}
