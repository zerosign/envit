use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Number {
    Integer(i64),
    Double(f64),
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Number(Number),
    String(String),
    Bool(bool),
    Optional(Option<Box<Value>>),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}
