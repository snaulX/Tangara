#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

pub enum Visibility {
    Private,
    Public,
    Protected,
    Internal
}

pub enum Value {
    Bool(bool),
    Number,
    String(String),
}

pub struct Attribute {
    name: String,
    values: [Value]
}

#[derive(Serialize, Deserialize)]
pub struct Package {
    name: String,
    id: u64
}

pub struct Class {
    vis: Visibility,
    is_static: bool,
    namespace: String,
    name: String,
    id: u64
}