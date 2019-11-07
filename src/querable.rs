use crate::querable::{error::Error, kind::QueryKind, types::Queryable};
use crate::value::Value;

impl Queryable for Value {
    #[inline]
    fn query_kind(&self) -> Option<QueryKind> {
        match self {
            Value::Literal(_) => None,
            Value::Array(_) => Some(QueryKind::Array),
            Value::Object(_) => Some(QueryKind::Dictionary),
        }
    }

    fn query_dict(&self, path: &str) -> Result<Self, Error> {
        match self {
            Value::Object(d) => d
                .get(path)
                .map(|v| v.clone())
                .ok_or(Error::KeyNotExist(String::from(path))),
            Value::Array(_) => Err(Error::TypeError(
                String::from(path),
                QueryKind::Array,
                QueryKind::Dictionary,
            )),
            _ => Err(Error::UnknownType(String::from(path))),
        }
    }

    fn query_array(&self, idx: usize) -> Result<Self, Error> {
        match self {
            Value::Array(d) => d
                .get(idx)
                .map(|v| v.clone())
                .ok_or(Error::IndexNotExist(idx)),
            Value::Object(_) => Err(Error::TypeError(
                format!("[{}]", idx),
                QueryKind::Dictionary,
                QueryKind::Array,
            )),
            _ => Err(Error::UnknownType(format!("[{}]", idx))),
        }
    }
}
