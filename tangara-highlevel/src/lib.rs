use std::collections::HashMap;
#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

#[cfg(feature = "builder")]
pub mod builder;

// Enums block
#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Visibility {
    Private,
    Public,
    Protected,
    Internal
}

#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub enum ArgumentKind {
    Default,
    DefaultValue(Value),
    Out,
    Ref
}

#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub enum TypeKind {
    Class(
        /// Sealed or not
        bool,
        /// Constructors
        Vec<Constructor>,
        /// Properties
        Vec<Property>,
        /// Methods
        Vec<Method>,
        /// Parents
        Vec<TypeRef>
    ),
    Enum(HashMap<String, Value>),
    Interface(
        /// Properties
        Vec<Property>,
        /// Methods
        Vec<Method>,
        /// Parents
        Vec<TypeRef>
    ),
    /// Data type that contains only data
    Struct(Vec<Constructor>, Vec<Property>),
    TypeAlias(Box<TypeRef>)
}

// TypeRef block
#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub enum TypeRef {
    Name(String),
    Id(u64),
    Direct(Type),

    Generic(Box<TypeRef>, Vec<TypeRef>),
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
#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
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
    Array(Vec<Value>),
    Tuple(Vec<Value>),
    Object(HashMap<String, Box<Value>>)
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

impl From<usize> for Value {
    fn from(value: usize) -> Self {
        if usize::BITS == 32 {
            Value::UInt(value as u32)
        } else {
            Value::ULong(value as u64)
        }
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
#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Argument(pub Vec<Attribute>, pub TypeRef, pub String, pub ArgumentKind);

/// Metadata for reflection's member (such as package, type, it's member and etc.)
#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Attribute(pub TypeRef, pub Vec<Value>);

/// Representation of generics
/// **Generics.0** - vec of types: `<A, B, C, D, ...>`
/// **Generics.1** - vec of 'where' statement with pairs like: `where String: TypeRef`
#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Generics(pub Vec<String>, pub Vec<(String, TypeRef)>);

// TODO add coding conventions field (naming conventions)
#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Package {
    pub attrs: Vec<Attribute>,
    pub name: String,
    pub id: u64,
    pub types: Vec<Type>
}

// TODO add operator overloading (including explicit/implicit convertation)
// TODO add abstract, static classes
#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Type {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub namespace: String,
    pub name: String,
    pub id: u64,
    pub generics: Generics,
    pub kind: TypeKind
}

#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Constructor {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub args: Vec<Argument>
}

// TODO add "init" - setter only in constructor
#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Property {
    pub attrs: Vec<Attribute>,
    pub getter_visibility: Visibility,
    pub setter_visibility: Option<Visibility>,
    pub prop_type: TypeRef,
    pub name: String,
    pub id: u64
}

// TODO add static, abstract, virtual methods
#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Method {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub name: String,
    pub id: u64,
    pub generics: Generics,
    pub args: Vec<Argument>,
    pub return_type: Option<TypeRef>
}