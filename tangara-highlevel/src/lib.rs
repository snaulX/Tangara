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
    Class(Vec<Constructor>, Vec<Property>),
    Enum(HashMap<String, Value>),
    Interface()
}

#[derive(Debug, Clone)]
pub enum TypeRef {
    Name(String),
    Id(u64),
    Direct(Type)
}

pub fn typeref_by_name(name: &str) -> TypeRef {
    TypeRef::Name(name.to_string())
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
pub struct Argument(TypeRef, String, Option<Value>);

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

#[derive(Debug, Clone)]
pub struct Constructor {
    pub vis: Visibility,
    pub args: Vec<Argument>
}

#[derive(Debug, Clone)]
pub struct Property {
    pub getter_visibility: Visibility,
    pub setter_visibility: Option<Visibility>,
    pub prop_type: TypeRef,
    pub name: String
}