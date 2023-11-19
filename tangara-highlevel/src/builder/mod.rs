use std::cell::RefCell;
use std::rc::Rc;
use crate::{Argument, Attribute, Package, Type, TypeRef, Visibility};
use xxhash_rust::const_xxh3::const_custom_default_secret;
use xxhash_rust::xxh3::xxh3_64_with_secret;

mod enum_builder;
mod class_builder;
mod struct_builder;
mod interface_builder;
mod alias_builder;
mod constructor_builder;
mod property_builder;
mod method_builder;

pub use crate::builder::alias_builder::*;
pub use crate::builder::class_builder::*;
pub use crate::builder::enum_builder::*;
pub use crate::builder::interface_builder::*;
pub use crate::builder::struct_builder::*;
pub use crate::builder::method_builder::*;
pub use crate::builder::property_builder::*;
pub use crate::builder::constructor_builder::*;

const PACKAGE_SECRET: [u8; 192] = const_custom_default_secret(772);
const TYPE_SECRET: [u8; 192] = const_custom_default_secret(4900);
const MEMBER_SECRET: [u8; 192] = const_custom_default_secret(18257);

/// Generate XXHash id for type with given name
pub(crate) fn generate_type_id(name: &String) -> u64 {
    xxh3_64_with_secret(name.as_bytes(), &TYPE_SECRET)
}

/// Generate XXHash id for property with given name
pub(crate) fn generate_property_id(name: &String) -> u64 {
    xxh3_64_with_secret(name.as_bytes(), &MEMBER_SECRET)
}

/// Generate vec of bytes made from type's id or collection of ids
fn get_typeref_bytes(type_ref: &TypeRef) -> Vec<u8> {
    match type_ref {
        TypeRef::Name(name) => {
            // NOTE: if name doesn't contains namespace (it would in most cases) it can get wrong id
            generate_type_id(&name).to_be_bytes().to_vec()
        }
        TypeRef::Id(id) => {
            id.to_be_bytes().to_vec()
        }
        TypeRef::Direct(t) => {
            t.id.to_be_bytes().to_vec()
        }
        TypeRef::Tuple(types) => {
            let mut bytes_slice = Vec::with_capacity(types.len());
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
            let mut bytes_slice = Vec::with_capacity(arg_types.len() + 1);
            bytes_slice.push(ret_bytes);
            for t in arg_types {
                bytes_slice.push(get_typeref_bytes(t));
            }
            bytes_slice.concat()
        }
    }
}

/// Generate XXHash id for method with given name
pub(crate) fn generate_method_id(name: &String, args: &Vec<Argument>) -> u64 {
    // What makes method unique? His name and types of his arguments
    let mut args_bytes = vec![];
    for arg in args {
        let arg_slice = get_typeref_bytes(&arg.1);
        args_bytes = [args_bytes, arg_slice].concat()
    }
    xxh3_64_with_secret([name.as_bytes(), args_bytes.as_slice()].concat().as_slice(), &MEMBER_SECRET)
}

pub trait TypeBuilder {
    fn add_attribute(&mut self, attr: Attribute) -> &mut Self;
    fn get_type(&self) -> Type;
    fn build(self) -> Type;
}

pub struct PackageBuilder {
    name: String,
    namespace: String,
    type_visibility: Visibility,
    constructor_visibility: Visibility,
    property_visibility: Visibility,
    method_visibility: Visibility,
    attrs: Vec<Attribute>,
    types: Vec<Type>
}

impl PackageBuilder {
    pub fn new(name: &str) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(
            Self {
                name: name.to_string(),
                namespace: name.to_string(),
                type_visibility: Visibility::Public,
                constructor_visibility: Visibility::Public,
                property_visibility: Visibility::Public,
                method_visibility: Visibility::Public,
                attrs: vec![],
                types: vec![]
            }
        ))
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn set_namespace(&mut self, namespace: &str) -> &mut Self {
        self.namespace = namespace.to_string();
        self
    }

    pub fn get_namespace(&self) -> String {
        self.namespace.clone()
    }

    pub fn get_id(&self) -> u64 {
        xxh3_64_with_secret(self.name.as_bytes(), &PACKAGE_SECRET)
    }

    pub fn add_attribute(&mut self, attr: Attribute) -> &mut Self {
        self.attrs.push(attr);
        self
    }

    pub fn add_type(&mut self, t: Type) -> &mut Self {
        self.types.push(t);
        self
    }

    pub fn build(&self) -> Package {
        Package {
            attrs: self.attrs.to_vec(),
            name: self.name.clone(),
            id: self.get_id(),
            types: self.types.to_vec(),
        }
    }
}

pub fn create_class(pkg_builder: Rc<RefCell<PackageBuilder>>, name: &str) -> ClassBuilder {
    ClassBuilder::new(pkg_builder, name)
}

pub fn create_struct(pkg_builder: Rc<RefCell<PackageBuilder>>, name: &str) -> StructBuilder {
    StructBuilder::new(pkg_builder, name)
}

pub fn create_enum(pkg_builder: Rc<RefCell<PackageBuilder>>, name: &str) -> EnumBuilder {
    EnumBuilder::new(pkg_builder, name)
}

pub fn create_interface(pkg_builder: Rc<RefCell<PackageBuilder>>, name: &str) -> InterfaceBuilder {
    InterfaceBuilder::new(pkg_builder, name)
}

pub fn create_alias(pkg_builder: Rc<RefCell<PackageBuilder>>, name: &str, alias: TypeRef) -> TypeAliasBuilder {
    TypeAliasBuilder::new(pkg_builder, name, alias)
}