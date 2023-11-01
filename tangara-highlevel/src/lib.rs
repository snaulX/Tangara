use std::collections::HashMap;
#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

#[cfg(feature = "builder")]
pub mod builder;

// Enums block
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Visibility {
    Private,
    Public,
    Protected,
    Internal
}

#[derive(Debug, Clone)]
pub enum ArgumentKind {
    Default,
    DefaultValue(Value),
    Out,
    Ref
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    Class(Vec<Constructor>, Vec<Property>, Vec<Method>),
    Enum(HashMap<String, Value>),
    Interface(Vec<Property>, Vec<Method>),
    // TODO: do something with Struct or remove it because it tries to full copy of Class
    Struct(Vec<Constructor>, Vec<Property>),
    TypeAlias(Box<TypeRef>)
}

// TypeRef block
#[derive(Debug, Clone)]
pub enum TypeRef {
    Name(String),
    Id(u64),
    Direct(Type),

    Tuple(Vec<TypeRef>),
    Fn(Option<Box<TypeRef>>, Vec<TypeRef>)
}

impl From<String> for TypeRef {
    fn from(value: String) -> Self {
        TypeRef::Name(value)
    }
}

impl From<&str> for TypeRef {
    fn from(value: &str) -> Self {
        TypeRef::Name(value.to_string())
    }
}

impl From<u64> for TypeRef {
    fn from(value: u64) -> Self {
        TypeRef::Id(value)
    }
}

impl From<Type> for TypeRef {
    fn from(value: Type) -> Self {
        TypeRef::Direct(value)
    }
}

// Value block
#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Byte(u8),
    Short(i16),
    Int(i32),
    Long(i64),
    SByte(i8),
    UShort(u16),
    UInt(u32),
    ULong(u64),
    String(String),
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Value::Byte(value)
    }
}

impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Value::Short(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Int(value)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Long(value)
    }
}

impl From<i8> for Value {
    fn from(value: i8) -> Self {
        Value::SByte(value)
    }
}

impl From<u16> for Value {
    fn from(value: u16) -> Self {
        Value::UShort(value)
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Value::UInt(value)
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Value::ULong(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.to_string())
    }
}

// Structs block
#[derive(Debug, Clone)]
pub struct Argument(Vec<Attribute>, TypeRef, String, ArgumentKind);

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
    pub name: String,
    pub id: u64
}

#[derive(Debug, Clone)]
pub struct Method {
    pub vis: Visibility,
    pub name: String,
    pub id: u64,
    pub args: Vec<Argument>,
    pub return_type: Option<TypeRef>
}