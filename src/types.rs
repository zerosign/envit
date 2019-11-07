//! # Overview
//!
//! Most of the types in here are public only to crates, except [`Parser`](Parser).
//!
use std::{
    borrow::Cow,
    cmp::Ordering,
    collections::{binary_heap::Iter, BinaryHeap},
    fmt,
    iter::FromIterator,
};

pub trait Parser {
    type Item: Sized;
    type Error: fmt::Debug;

    fn parse(raw: &str) -> Result<Self::Item, Self::Error>;
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Pair {
    pub fields: Vec<String>,
    pub value: String,
}

impl PartialOrd for Pair {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Pair {
    #[inline]
    pub fn new<'a, I>(l: I, r: &'a str) -> Self
    where
        I: Iterator<Item = &'a str>,
    {
        Pair {
            fields: l.map(String::from).collect(),
            value: String::from(r),
        }
    }
}

impl Ord for Pair {
    fn cmp(&self, other: &Self) -> Ordering {
        // length > natural string order
        self.fields.cmp(&other.fields)
    }
}

#[derive(Debug)]
pub(crate) struct PairSeq {
    inner: BinaryHeap<Pair>,
}

impl<I> From<I> for PairSeq
where
    I: Iterator<Item = Pair>,
{
    #[inline]
    fn from(iter: I) -> Self {
        PairSeq {
            inner: BinaryHeap::from_iter(iter),
        }
    }
}

impl PairSeq {
    #[inline]
    pub(crate) fn iter(&self) -> Iter<Pair> {
        self.inner.iter()
    }
}
