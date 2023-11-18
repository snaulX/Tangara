use std::cell::RefCell;
use std::rc::Rc;
use crate::{Attribute, Package, Type, TypeRef, Visibility};
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

pub use crate::builder::alias_builder::TypeAliasBuilder;
pub use crate::builder::class_builder::ClassBuilder;
pub use crate::builder::enum_builder::EnumBuilder;
pub use crate::builder::interface_builder::InterfaceBuilder;
pub use crate::builder::struct_builder::StructBuilder;

const PACKAGE_SECRET: [u8; 192] = const_custom_default_secret(772);
const TYPE_SECRET: [u8; 192] = const_custom_default_secret(4900);
const MEMBER_SECRET: [u8; 192] = const_custom_default_secret(18257);

/// Generate XXHash id for type with given name
pub(crate) fn generate_type_id(name: &String) -> u64 {
    xxh3_64_with_secret(name.as_bytes(), &TYPE_SECRET)
}

/// Generate XXHash id for member with given name
pub(crate) fn generate_member_id(name: &String) -> u64 {
    xxh3_64_with_secret(name.as_bytes(), &MEMBER_SECRET)
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