//! Overview
//!
//! Most of the types in here are public only to crates, except [`Parser`](Parser).
//!

use std::{
    cmp::{Ord, Ordering, PartialOrd},
    collections::{binary_heap::Iter, hash_map::DefaultHasher, BinaryHeap, HashMap},
    hash::{Hash, Hasher},
    iter::FromIterator,
    str::pattern::Pattern,
};

// pub type Key<'b> = Vec<&'b str>;
type IndexRef = Vec<usize>;

// (index of indices, index of tree)
#[derive(Debug, Copy, Clone)]
pub struct Index {
    node: usize,
    level: usize,
}

impl Default for Index {
    #[inline]
    fn default() -> Self {
        Self { node: 0, level: 0 }
    }
}

impl Index {
    #[inline]
    pub fn new() -> Self {
        Self { node: 0, level: 0 }
    }

    #[inline]
    pub fn up(&self) -> Self {
        Self {
            node: self.node,
            level: self.level - 1,
        }
    }

    #[inline]
    pub fn down(&self) -> Self {
        Self {
            node: self.node,
            level: self.level + 1,
        }
    }

    #[inline]
    pub fn next(&self) -> Self {
        Self {
            node: self.node + 1,
            level: 0,
        }
    }
}

pub enum Kind {
    Leaf,
    Node,
}

#[derive(Debug, Clone)]
pub struct StringDict<'a> {
    reverse: Vec<&'a str>,
    indices: Vec<IndexRef>,
    data: Vec<&'a str>,
}

impl<'a> Default for StringDict<'a> {
    #[inline]
    fn default() -> Self {
        Self {
            reverse: Vec::with_capacity(0),
            indices: Vec::with_capacity(0),
            data: Vec::with_capacity(0),
        }
    }
}

pub enum Cursor {
    // no available cursor for next iterator
    Unavailable,
    // next cursor will be a Node (could be another parent Node or Leaf)
    Node,
    // next cursor will be a new node (parent)
    Parent,
}

impl<'a> StringDict<'a> {
    #[inline]
    pub fn hash_parent(&self, idx: Index) -> Option<u64> {
        self.indices.get(idx.node).map(|d| {
            // memoize this?
            let mut hasher = DefaultHasher::new();
            // create hash for this stringdict
            Hash::hash(&d.get(0..idx.level - 1), &mut hasher);
            hasher.finish()
        })
    }

    #[inline]
    pub fn is_available(&self, idx: Index) -> bool {
        self.indices.get(idx.node).is_some()
    }

    #[inline]
    pub fn fetch_index_kind(&self, index: Index) -> Option<Kind> {
        match self.indices.get(index.node) {
            Some(leaf) => {
                if index.level == leaf.len() - 1 {
                    Some(Kind::Leaf)
                } else if index.level < leaf.len() - 1 && index.level >= 0 {
                    Some(Kind::Node)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    #[inline]
    pub fn fetch_value(&self, index: Index) -> Option<String> {
        self.data.get(index.node).map(move |s| String::from(*s))
    }

    ///
    /// returns min heap
    ///
    #[inline]
    fn parse_lines<'b, P: Pattern<'b> + Copy, PS: Pattern<'b> + Copy>(
        raw: &'b str,
        kv_sep: P,
        key_sep: PS,
    ) -> BinaryHeap<KeyValue<'_>> {
        BinaryHeap::from_iter(raw.split('\n').filter_map(move |line| {
            let line = line.trim();

            if !line.starts_with('#') {
                // split for the first found 'kv_sep'.
                let pair = line.splitn(2, kv_sep).collect::<Vec<_>>();

                match &pair[..] {
                    &[key, value] => {
                        let fields = key.split(key_sep).collect::<Vec<_>>();
                        Some(KeyValue { fields, value })
                    }
                    _ => None,
                }
            } else {
                None
            }
        }))
    }

    fn from_pairs(iter: Iter<KeyValue<'a>>) -> Self {
        let mut inner = Self::default();
        let mut reverse_idx = HashMap::<&'_ str, usize>::new();

        let mut idx = 0;
        for KeyValue { fields, value } in iter {
            let mut indices: IndexRef = IndexRef::new();

            for field in fields {
                // when key found
                let ridx = match reverse_idx.get(field) {
                    None => {
                        let old_idx = idx;
                        reverse_idx.insert(field, idx);
                        inner.reverse.push(field);
                        idx += 1;
                        old_idx
                    }
                    Some(ridx) => *ridx,
                };

                indices.push(ridx);
            }
            inner.indices.push(indices);
            inner.data.push(value);
        }
        inner
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct KeyValue<'a> {
    fields: Vec<&'a str>,
    value: &'a str,
}

#[inline]
fn diff<'a, 'b>(lhs: &Vec<&'a str>, rhs: &Vec<&'b str>) -> isize {
    (rhs.len() as isize) - (lhs.iter().zip(rhs.iter()).filter(|(l, r)| l != r).count() as isize)
}

impl<'a> Ord for KeyValue<'a> {
    fn cmp(&self, other: &KeyValue<'_>) -> Ordering {
        let diff = diff(&self.fields, &other.fields);

        if diff < 0 {
            Ordering::Less
        } else if diff > 0 {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl<'a> PartialOrd for KeyValue<'a> {
    fn partial_cmp(&self, other: &KeyValue<'_>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[test]
fn test_parse_lines() {
    //
    // I want to reorder
    // CONFIG -> DATABASE -> DATABASE_NAME -> DATABASE_USERNAME -> DATABASE_CONNECTION
    //

    let raw = r#"CONFIG__DATABASE__NAME=name
CONFIG__DATABASE__CREDENTIAL__TYPE=password
CONFIG__DATABASE__CREDENTIAL__PASSWORD=some_password
CONFIG__DATABASE__CONNECTION__POOL=10
CONFIG__DATABASE__USERNAME=username
CONFIG__DATABASE__CONNECTION__TIMEOUT=10
CONFIG__DATABASE__CONNECTION__RETRIES=10,20,30
# CONFIG__APPLICATION__ENV=development
CONFIG__APPLICATION__LOGGER__LEVEL=info"#;

    let expected = vec![
        KeyValue {
            fields: vec!["CONFIG", "DATABASE", "NAME"],
            value: "name",
        },
        KeyValue {
            fields: vec!["CONFIG", "DATABASE", "CREDENTIAL", "TYPE"],
            value: "password",
        },
        KeyValue {
            fields: vec!["CONFIG", "DATABASE", "CREDENTIAL", "PASSWORD"],
            value: "some_password",
        },
        KeyValue {
            fields: vec!["CONFIG", "DATABASE", "CONNECTION", "POOL"],
            value: "10",
        },
        KeyValue {
            fields: vec!["CONFIG", "DATABASE", "USERNAME"],
            value: "username",
        },
        KeyValue {
            fields: vec!["CONFIG", "DATABASE", "CONNECTION", "TIMEOUT"],
            value: "10",
        },
        KeyValue {
            fields: vec!["CONFIG", "DATABASE", "CONNECTION", "RETRIES"],
            value: "10,20,30",
        },
        KeyValue {
            fields: vec!["CONFIG", "APPLICATION", "LOGGER", "LEVEL"],
            value: "info",
        },
    ];

    let heap = StringDict::parse_lines(raw, '=', "__");
    for (idx, item) in heap.iter().enumerate() {
        assert_eq!(&expected[idx], item);
        println!("item: {:?}", item);
    }

    let dict = StringDict::from_pairs(heap.iter());
    println!("result: {:?}", dict);
}
