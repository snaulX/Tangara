use std::cell::RefCell;
use std::rc::Rc;
use crate::{Attribute, generate_package_id, NamingConventions, Package, Type, TypeRef, Visibility};

mod enum_builder;
mod class_builder;
mod struct_builder;
mod interface_builder;
mod alias_builder;
mod constructor_builder;
mod property_builder;
mod method_builder;
mod field_builder;

pub use crate::builder::alias_builder::*;
pub use crate::builder::class_builder::*;
pub use crate::builder::enum_builder::*;
pub use crate::builder::interface_builder::*;
pub use crate::builder::struct_builder::*;
pub use crate::builder::method_builder::*;
pub use crate::builder::property_builder::*;
pub use crate::builder::constructor_builder::*;
pub use crate::builder::field_builder::*;

pub trait TypeBuilder {
    fn add_attribute(&mut self, attr: Attribute) -> &mut Self;
    fn set_visibility(&mut self, vis: Visibility) -> &mut Self;
    fn get_type(&self) -> Type;
    fn build(self) -> Type;
}

pub trait GenericsCollector {
    fn generic(&mut self, generic: String) -> &mut Self;
    fn generic_where(&mut self, statement: (String, TypeRef)) -> &mut Self;
    fn generics(&mut self, generics: Vec<String>) -> &mut Self {
        for g in generics {
            self.generic(g);
        }
        self
    }
    fn generic_wheres(&mut self, generic_wheres: Vec<(String, TypeRef)>) -> &mut Self {
        for gw in generic_wheres {
            self.generic_where(gw);
        }
        self
    }
}

pub trait AttributeCollector {
    fn add_attribute(&mut self, attribute: Attribute) -> &mut Self;
}

impl<T: TypeBuilder> AttributeCollector for T {
    #[inline]
    fn add_attribute(&mut self, attribute: Attribute) -> &mut Self {
        self.add_attribute(attribute)
    }
}

pub struct PackageBuilder {
    name: String,
    namespace: String,
    /// Default visibility for types
    pub type_visibility: Visibility,
    /// Default visibility for constructors
    pub constructor_visibility: Visibility,
    /// Default visibility for properties/variants/fields
    pub member_visibility: Visibility,
    /// Default visibility for methods
    pub method_visibility: Visibility,
    attrs: Vec<Attribute>,
    types: Vec<Type>,
    naming: NamingConventions
}

impl PackageBuilder {
    pub fn new(name: &str, naming: NamingConventions) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(
            Self {
                name: name.to_string(),
                namespace: naming.namespace.from(name, &naming.package).unwrap(),
                type_visibility: Visibility::Public,
                constructor_visibility: Visibility::Public,
                member_visibility: Visibility::Public,
                method_visibility: Visibility::Public,
                attrs: vec![],
                types: vec![],
                naming
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

    pub fn set_naming(&mut self, naming: NamingConventions) -> &mut Self {
        self.name = naming.package.from(&self.name, &self.naming.package).unwrap();
        self.namespace = naming.namespace.from(&self.namespace, &self.naming.namespace).unwrap();
        // TODO change types naming
        self.naming = naming;
        self
    }

    pub fn get_naming(&self) -> NamingConventions {
        self.naming.clone()
    }

    pub fn get_id(&self) -> u64 {
        generate_package_id(&self.name)
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
            naming: self.naming.clone()
        }
    }
}

impl AttributeCollector for PackageBuilder {
    fn add_attribute(&mut self, attr: Attribute) -> &mut Self {
        self.attrs.push(attr);
        self
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

pub fn create_enum_class(pkg_builder: Rc<RefCell<PackageBuilder>>, name: &str) -> EnumClassBuilder {
    EnumClassBuilder::new(pkg_builder, name)
}

pub fn create_interface(pkg_builder: Rc<RefCell<PackageBuilder>>, name: &str) -> InterfaceBuilder {
    InterfaceBuilder::new(pkg_builder, name)
}

pub fn create_alias(pkg_builder: Rc<RefCell<PackageBuilder>>, name: &str, alias: TypeRef) -> TypeAliasBuilder {
    TypeAliasBuilder::new(pkg_builder, name, alias)
}