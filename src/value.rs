//!
//!
//!
//!

use std::collections::BTreeMap;

#[derive(Debug, PartialOrd, PartialEq)]
pub enum Number {
    Integer(i64),
    Double(f64),
}

#[derive(Debug, PartialOrd, PartialEq)]
pub enum Value {
    Number(Number),
    String(String),
    Bool(bool),
    Optional(Option<Box<Value>>),
    Array(Vec<Value>),
    Object(Box<BTreeMap<String, Value>>),
}
