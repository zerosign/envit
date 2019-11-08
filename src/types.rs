//! Overview
//!
//! Most of the types in here are public only to crates, except [`Parser`](Parser).
//!
use std::{borrow::Cow, cmp::Ordering, collections::BinaryHeap, fmt, iter::FromIterator};

///
/// Parser trait for this crates.
///
/// Don't expose Parser trait into public API since
/// it doesn't have good simetry on parsing an Object.
///
pub(crate) trait Parser {
    type Item: Sized;
    type Error: fmt::Debug;

    fn parse(raw: &str) -> Result<Self::Item, Self::Error>;
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Pair<'a> {
    pub fields: Cow<'a, [String]>,
    pub value: Cow<'a, str>,
}

impl<'a> Pair<'a> {
    pub fn new<I, S>(fields: I, value: S) -> Self
    where
        I: Iterator<Item = String>,
        S: Into<Cow<'a, str>>,
    {
        Self {
            fields: Cow::from(fields.collect::<Vec<_>>()),
            value: S::into(value),
        }
    }
}

impl<'a> PartialOrd for Pair<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for Pair<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        // length > natural string order
        self.fields.cmp(&other.fields)
    }
}

#[derive(Debug)]
pub(crate) struct PairSeq<'a> {
    inner: BinaryHeap<Pair<'a>>,
}

impl<'a, I> From<I> for PairSeq<'a>
where
    I: Iterator<Item = Pair<'a>>,
{
    #[inline]
    fn from(iter: I) -> Self {
        PairSeq {
            inner: BinaryHeap::from_iter(iter),
        }
    }
}

impl<'a> IntoIterator for PairSeq<'a> {
    type Item = Pair<'a>;
    type IntoIter = ::std::collections::binary_heap::IntoIter<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}
