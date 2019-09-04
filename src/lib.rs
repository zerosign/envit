use std::{
    collections::BTreeMap,
    io::{self, BufRead},
};

#[derive(Debug, PartialOrd, PartialEq)]
pub enum Value {
    Integer(i64),
    Double(f64),
    String(String),
    Bool(bool),
    Optional(Option<Box<Value>>),
    Array(Vec<Value>),
    Object(Box<BTreeMap<String, Value>>),
}

pub type Pair<T> = (T, String);

#[derive(Debug)]
pub enum PairError {
    EmptyPair,
    IncompletePair,
    SizeError,
}

#[derive(Debug)]
pub enum Error {
    ParseError(io::Error),
    PairError(PairError),
}

pub fn parse_env<'a, R>(reader: R) -> Vec<Result<Pair<Vec<String>>, Error>>
where
    R: BufRead,
{
    reader
        .lines()
        .map(move |r| r.map_err(|e| Error::ParseError(e)))
        .map(move |r| r.map(move |line| String::from(line.trim())))
        .filter(|r| {
            r.as_ref()
                .map(|line| !line.is_empty() && !line.starts_with('#'))
                .unwrap_or(false)
        })
        // Iter<Result<String>>
        .map(move |r| {
            r.and_then(|line| {
                let pair = line.split('=').collect::<Vec<&str>>();

                match *pair {
                    [] => Err(Error::PairError(PairError::EmptyPair)),
                    [_] => Err(Error::PairError(PairError::IncompletePair)),
                    [key, value] => {
                        let fields = key
                            .split("__")
                            .map(move |line| line.to_lowercase())
                            .collect::<Vec<String>>();

                        Ok((fields, String::from(value)))
                    }
                    _ => Err(Error::PairError(PairError::SizeError)),
                }
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{parse_env, Value};
    use std::{collections::BTreeMap, io::Cursor};

    #[test]
    fn parse_simple_envs() {
        let raw = r#"
    CONFIG__DATABASE__NAME=name
    CONFIG__DATABASE__USERNAME=username
    CONFIG__DATABASE__CREDENTIAL__TYPE=password
    CONFIG__DATABASE__CREDENTIAL__PASSWORD=some_password
    CONFIG__DATABASE__CONNECTION__POOL=10
    CONFIG__DATABASE__CONNECTION__TIMEOUT=10
    CONFIG__DATABASE__CONNECTION__RETRIES=10,20,30
    CONFIG__APPLICATION__ENV=development
    CONFIG__APPLICATION__LOGGER__LEVEL=info"#;

        let reader = Cursor::new(raw);

        let results = parse_env(reader);
        let mut root = BTreeMap::new();

        // results.fold(|| {})

        // println!("results : {:?}", results);
    }
}
