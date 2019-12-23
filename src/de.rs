use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, HashSet},
    io::{BufRead, Cursor},
    iter::FromIterator,
};

#[derive(Debug, Clone)]
pub(crate) struct EnvPair {
    fields: Vec<String>,
    value: String,
}

impl EnvPair {
    #[inline]
    pub fn from_str<'a>(line: &'a str, comment: char, kv_sep: char, key_sep: &str) -> Option<Self> {
        let line = line.trim();

        if line.starts_with(comment) {
            return None;
        }

        let pair = line
            .splitn(2, kv_sep)
            .map(move |line| line.to_string())
            .collect::<Vec<_>>();

        match &pair[..] {
            [key, value] => {
                let fields = key
                    .split(key_sep)
                    .map(move |line| line.to_string())
                    .collect::<Vec<_>>();

                Some(Self {
                    fields: fields,
                    value: value.clone(),
                })
            }
            _ => None,
        }
    }
}

impl PartialEq for EnvPair {
    #[inline]
    fn eq(&self, other: &EnvPair) -> bool {
        self.fields.eq(&other.fields)
    }
}

impl Eq for EnvPair {}

impl PartialOrd for EnvPair {
    fn partial_cmp(&self, other: &EnvPair) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EnvPair {
    fn cmp(&self, other: &EnvPair) -> Ordering {
        self.fields.cmp(&other.fields)
    }
}

/// This abstraction give us contract that
/// if the parent not exists, we could create only the branch
/// This save us to do combinatoric search for a parent sets
/// in the segment
///
/// the parent shouldn't be empty even it's first node.
///
/// [0, 1, 2] -> Segment { parent : [0, 1], branch: [2] }
///
/// we could savely said that if hole is exists, then create a new node based on it.
/// when traversing Segment will gradually move hole to parent.
///
/// example:
///
/// [[0, 1, 2], [0, 1, 3], [0, 1, 4, 5], [0, 1, 4, 6], [0, 1, 7, 8], [0, 1, 7, 9], [0, 1, 7, 10], [0, 11, 12, 13]]
///
/// hole: [0, 1], parent: [], leaf: 2
/// hole: [], parent: [0, 1], leaf: 3
/// hole: [4], parent: [0, 1], leaf: 5
/// hole: [], parent: [0, 1, 4], leaf: 6
/// hole: [7], parent: [0, 1], leaf: 8
/// hole: [], parent: [0, 1, 7], leaf: 9
/// hole: [], parent: [0, 1, 7], leaf: 10
/// hole: [11, 12], parent: [0], leaf: 13
///
#[derive(Debug, Clone)]
pub(crate) struct Segment {
    hole: Vec<usize>,
    parent: Vec<usize>,
    leaf: usize,
}

#[derive(Debug, Clone)]
pub struct Envs<'a> {
    reverse: Vec<&'a str>,
    indices: Vec<Segment>,
    data: Vec<&'a str>,
}

impl<'a> Default for Envs<'a> {
    #[inline]
    fn default() -> Self {
        Self {
            reverse: Vec::with_capacity(0),
            indices: Vec::with_capacity(0),
            data: Vec::with_capacity(0),
        }
    }
}

impl<'a> Envs<'a> {
    #[inline]
    pub fn from_str<'c>(
        raw: &'c str,
        comment: char,
        kv_sep: char,
        key_sep: &str,
    ) -> Result<Self, ()> {
        Self::from_reader(Cursor::new(raw), comment, kv_sep, key_sep)
    }

    pub fn from_reader<'c, R>(
        reader: R,
        comment: char,
        kv_sep: char,
        key_sep: &str,
    ) -> Result<Self, ()>
    where
        R: BufRead,
    {
        let data: BinaryHeap<EnvPair> = BinaryHeap::from_iter(reader.lines().filter_map(|r| {
            r.ok()
                .as_ref()
                .and_then(move |line| EnvPair::from_str(line, comment, kv_sep, key_sep))
        }));

        let mut inner = Self::default();
        let mut reverse_idx = HashMap::<String, usize>::new();
        //
        // env pair fields will always direct to a leaf node in the branch/tree
        // so, the only possible parent node would be fields[0-k-1] or
        // any subsequences fields that exists in the parent sets
        //
        // logic:
        // sets : {}
        // fields : [0 ... ii]
        //
        // sets : {[0 ... ii-1]}
        // fields : [0 ... ii-1 ... k]
        //
        // sets : {[0 ... ii-1], [ii-1 ... k-1], [0 ... k-1]}
        //
        // example:
        // [[0, 1, 2], [0, 1, 3], [0, 1, 4, 5], [0, 1, 4, 6], [0, 1, 7, 8], [0, 1, 7, 9], [0, 1, 7, 10], [0, 11, 12, 13]]
        //
        //
        //
        let mut parents = HashSet::<Vec<usize>>::new();

        let mut idx = 0;
        for EnvPair { fields, value } in data.iter() {
            let mut indices: Vec<usize> = Vec::new();

            for field in fields {
                let ridx = match reverse_idx.get(field) {
                    Some(ridx) => *ridx,
                    _ => {
                        let old_idx = idx;
                        reverse_idx.insert(field.to_string(), idx);
                        inner.reverse.push(field);
                        idx += 1;
                        old_idx
                    }
                };

                indices.push(ridx);

                if !parents.contains(indices.as_slice()) {
                    parents.insert(indices.clone());
                }
            }

            // do combinatoric checks for each leftmost subsequences &
        }

        Err(())
    }
}
