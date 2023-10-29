use std::collections::HashMap;
#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

#[cfg(feature = "builder")]
pub mod builder;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Visibility {
    Private,
    Public,
    Protected,
    Internal
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    Class(),
    Enum(HashMap<String, Value>),
    Interface()
}

#[derive(Debug, Clone)]
pub enum TypeRef {
    Name(String),
    Id(u64),
    Direct(Type)
}

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Byte(u8),
    Short(i16),
    Int(i32),
    Long(i64),
    UShort(u16),
    UInt(u32),
    ULong(u64),
    String(String),
}

#[derive(Debug, Clone)]
pub struct Attribute(TypeRef, Vec<Value>);

//#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub id: u64,
    pub types: Vec<Type>
}

#[derive(Debug, Clone)]
pub struct Type {
    pub vis: Visibility,
    pub namespace: String,
    pub name: String,
    pub id: u64,
    pub attrs: Vec<Attribute>,
    pub kind: TypeKind
}