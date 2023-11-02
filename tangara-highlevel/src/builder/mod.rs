use crate::{Package, Type, TypeRef, Visibility};
use xxhash_rust::const_xxh3::const_custom_default_secret;
use xxhash_rust::xxh3::xxh3_64_with_secret;
use crate::builder::alias_builder::TypeAliasBuilder;
use crate::builder::class_builder::ClassBuilder;
use crate::builder::enum_builder::EnumBuilder;
use crate::builder::interface_builder::InterfaceBuilder;
use crate::builder::struct_builder::StructBuilder;
use crate::TypeKind::TypeAlias;

pub mod enum_builder;
pub mod class_builder;
pub mod struct_builder;
pub mod interface_builder;
pub mod alias_builder;
pub mod constructor_builder;
pub mod property_builder;
pub mod method_builder;

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
    fn get_type(&self) -> Type;
    fn build(&mut self) -> &mut PackageBuilder;
}

pub struct PackageBuilder {
    name: String,
    namespace: String,
    type_visibility: Visibility,
    constructor_visibility: Visibility,
    property_visibility: Visibility,
    method_visibility: Visibility,
    types: Vec<Type>
}

impl PackageBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            namespace: name.to_string(),
            type_visibility: Visibility::Public,
            constructor_visibility: Visibility::Public,
            property_visibility: Visibility::Public,
            method_visibility: Visibility::Public,
            types: Vec::new()
        }
    }

    pub fn set_namespace(&mut self, namespace: &str) -> &mut Self {
        self.namespace = namespace.to_string();
        self
    }

    pub fn create_class(&mut self, name: &str) -> ClassBuilder {
        ClassBuilder::new(self, name)
    }

    pub fn create_struct(&mut self, name: &str) -> StructBuilder {
        StructBuilder::new(self, name)
    }

    pub fn create_enum(&mut self, name: &str) -> EnumBuilder {
        EnumBuilder::new(self, name)
    }

    pub fn create_interface(&mut self, name: &str) -> InterfaceBuilder {
        InterfaceBuilder::new(self, name)
    }

    pub fn create_alias(&mut self, name: &str, alias: TypeRef) -> TypeAliasBuilder {
        TypeAliasBuilder::new(self, name, alias)
    }

    pub fn build(&self) -> Package {
        Package {
            name: self.name.clone(),
            id: xxh3_64_with_secret(self.name.as_bytes(), &PACKAGE_SECRET),
            types: self.types.to_vec(),
        }
    }
}