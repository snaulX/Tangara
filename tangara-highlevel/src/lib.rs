use std::collections::HashMap;
#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};
use xxhash_rust::const_xxh3::const_custom_default_secret;
use xxhash_rust::xxh3::xxh3_64_with_secret;

mod naming;
pub use naming::*;

#[cfg(feature = "builder")]
pub mod builder;

// Enums block
#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Ord, PartialOrd, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum Visibility {
    Private,
    Protected,
    Internal,
    Public
}

#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub enum ArgumentKind {
    /// Argument that copies itself and pass it into function
    Default,
    /// Argument which contains default value and can be skipped
    DefaultValue(Value),
    /// Argument which moves into function by reference and cannot be used there except being assigned
    Out,
    /// Argument which moves into function by reference and can be changed there
    Ref,
    /// Argument which moves into function by reference and cannot be changed there
    In
}

#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub enum TypeKind {
    Class {
        is_sealed: bool,
        constructors: Vec<Constructor>,
        properties: Vec<Property>,
        fields: Vec<Field>,
        static_properties: Vec<Property>,
        static_fields: Vec<Field>,
        methods: Vec<Method>,
        parents: Vec<TypeRef>
    },
    Enum {
        /// Variants: Name = Value
        variants: Vec<(String, Value)>
    },
    EnumClass {
        /// Variants: Name(Properties)
        variants: Vec<Variant>,
        methods: Vec<Method>
    },
    Interface {
        properties: Vec<Property>,
        methods: Vec<Method>,
        parents: Vec<TypeRef>
    },
    /// Type's kind that contains only data
    Struct {
        constructors: Vec<Constructor>,
        fields: Vec<Field>,
        static_fields: Vec<Field>
    },
    TypeAlias(Box<TypeRef>)
}

#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum MethodKind {
    /// Default method: with body, with `this`
    Default,
    /// Without body
    Abstract,
    /// With body but can be overrided from
    Virtual,
    /// With body but hasn't `this` field
    Static
}

// TypeRef block
#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub enum TypeRef {
    Name(String),
    Id(u64),

    Generic(Box<TypeRef>, Vec<TypeRef>),
    Tuple(Vec<TypeRef>),
    Fn(Option<Box<TypeRef>>, Vec<TypeRef>)
}

const PACKAGE_SECRET: [u8; 192] = const_custom_default_secret(772);
const TYPE_SECRET: [u8; 192] = const_custom_default_secret(4900);
const MEMBER_SECRET: [u8; 192] = const_custom_default_secret(18257);

/// Generate XXHash id for package with given name
pub fn generate_package_id(name: &str) -> u64 {
    xxh3_64_with_secret(name.as_bytes(), &PACKAGE_SECRET)
}

/// Generate XXHash id for type with given full name (don't forget about namespace)
pub fn generate_type_id(name: &str) -> u64 {
    xxh3_64_with_secret(name.as_bytes(), &TYPE_SECRET)
}

/// Generate XXHash id for property/variant with given name
pub fn generate_member_id(name: &str) -> u64 {
    xxh3_64_with_secret(name.as_bytes(), &MEMBER_SECRET)
}

/// Generate vec of bytes made from type's id or collection of ids
pub fn get_typeref_bytes(type_ref: &TypeRef) -> Vec<u8> {
    match type_ref {
        TypeRef::Name(name) => {
            // NOTE: if name doesn't contains namespace (it would in most cases) it can get wrong id
            generate_type_id(&name).to_be_bytes().to_vec()
        }
        TypeRef::Id(id) => {
            id.to_be_bytes().to_vec()
        }
        TypeRef::Generic(base_type, generics) => {
            let mut bytes_slice = Vec::with_capacity(generics.len() + 2);
            bytes_slice.push(vec![0]); // id of generic
            bytes_slice.push(get_typeref_bytes(&base_type));
            for g in generics {
                bytes_slice.push(get_typeref_bytes(g));
            }
            bytes_slice.concat()
        }
        TypeRef::Tuple(types) => {
            let mut bytes_slice = Vec::with_capacity(types.len() + 1);
            bytes_slice.push(vec![1]); // id of tuple
            for t in types {
                bytes_slice.push(get_typeref_bytes(t));
            }
            bytes_slice.concat()
        }
        TypeRef::Fn(return_type, arg_types) => {
            let ret_bytes = if let Some(ret_type) = return_type {
                get_typeref_bytes(ret_type)
            } else {
                // empty u64 id with all 0es
                vec![0u8; 8]
            };
            let mut bytes_slice = Vec::with_capacity(arg_types.len() + 2);
            bytes_slice.push(vec![2]); // id of fn
            bytes_slice.push(ret_bytes);
            for t in arg_types {
                bytes_slice.push(get_typeref_bytes(t));
            }
            bytes_slice.concat()
        }
    }
}

/// Generate XXHash id for method with given name
pub fn generate_method_id(name: &str, args: &Vec<Argument>) -> u64 {
    // What makes method unique? His name and types of his arguments
    let mut args_bytes = vec![];
    for arg in args {
        let arg_slice = get_typeref_bytes(&arg.1);
        args_bytes = [args_bytes, arg_slice].concat()
    }
    xxh3_64_with_secret([name.as_bytes(), args_bytes.as_slice()].concat().as_slice(), &MEMBER_SECRET)
}

impl PartialEq for TypeRef {
    fn eq(&self, other: &Self) -> bool {
        get_typeref_bytes(self) == get_typeref_bytes(other)
    }
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

impl From<&Type> for TypeRef {
    fn from(value: &Type) -> Self {
        TypeRef::Id(value.id)
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
    Float(f32),
    Double(f64),
    String(String),
    Array(Vec<Value>),
    Tuple(Vec<Value>),
    Object(HashMap<String, Box<Value>>)
}

// - ULong!!
// - No, UShort!
// *KABOOM*
// you know ULong/UShort sounds like "you long/short" and long and short are like.. uhm forget this

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

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::Float(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Double(value)
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

impl From<Property> for Argument {
    fn from(value: Property) -> Self {
        Argument(value.attrs, value.prop_type, value.name, ArgumentKind::Default)
    }
}

impl From<Field> for Argument {
    fn from(value: Field) -> Self {
        Argument(value.attrs, value.field_type, value.name, ArgumentKind::Default)
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

#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Package {
    pub attrs: Vec<Attribute>,
    pub name: String,
    pub id: u64,
    pub types: Vec<Type>,
    pub naming: NamingConventions
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

#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Field {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub field_type: TypeRef,
    pub name: String,
    pub default_value: Option<Value>,
    pub id: u64
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

#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Method {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub name: String,
    pub id: u64,
    pub generics: Generics,
    pub args: Vec<Argument>,
    pub return_type: Option<TypeRef>,
    pub kind: MethodKind
}

/// Rust-like variant for 'enum class', can contain fields
#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct Variant {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub name: String,
    pub id: u64,
    pub fields: Vec<Field>
}