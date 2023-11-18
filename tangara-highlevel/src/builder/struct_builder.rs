use std::cell::RefCell;
use std::rc::Rc;
use crate::builder::{generate_type_id, PackageBuilder, TypeBuilder};
use crate::{Attribute, Constructor, Property, Type, TypeRef, Visibility};
use crate::builder::constructor_builder::{ConstructorBuilder, ConstructorCollector};
use crate::builder::property_builder::{PropertyBuilder, PropertyCollector};
use crate::TypeKind::Struct;

pub struct StructBuilder {
    builder: Rc<RefCell<PackageBuilder>>,
    attrs: Vec<Attribute>,
    name: String,
    vis: Visibility,
    constructors: Vec<Constructor>,
    properties: Vec<Property>
}

impl StructBuilder {
    pub(crate) fn new(builder: Rc<RefCell<PackageBuilder>>, name: &str) -> Self {
        let vis = builder.borrow().type_visibility;
        Self {
            builder,
            attrs: vec![],
            name: name.to_string(),
            vis,
            constructors: Vec::new(),
            properties: Vec::new()
        }
    }

    pub fn set_visibility(&mut self, vis: Visibility) -> &mut Self {
        self.vis = vis;
        self
    }

    pub fn add_constructor(&mut self) -> ConstructorBuilder<Self> {
        ConstructorBuilder::new(self)
    }

    pub fn add_property(&mut self, prop_type: TypeRef, name: &str) -> PropertyBuilder<Self> {
        PropertyBuilder::new(self, prop_type, name)
    }
}

impl TypeBuilder for StructBuilder {
    fn add_attribute(&mut self, attr: Attribute) -> &mut Self {
        self.attrs.push(attr);
        self
    }

    fn get_type(&self) -> Type {
        Type {
            attrs: self.attrs.to_vec(),
            vis: self.vis.clone(),
            namespace: self.builder.borrow().namespace.clone(),
            name: self.name.clone(),
            id: generate_type_id(&self.name),
            kind: Struct(self.constructors.to_vec(), self.properties.to_vec())
        }
    }

    fn build(self) -> Type {
        let result_type = self.get_type();
        let mut builder = self.builder.borrow_mut();
        builder.add_type(result_type.clone());
        result_type
    }
}

impl ConstructorCollector for StructBuilder {
    fn get_default_visibility(&self) -> Visibility {
        self.builder.borrow().constructor_visibility
    }

    fn add_constructor(&mut self, constructor: Constructor) {
        self.constructors.push(constructor)
    }
}

impl PropertyCollector for StructBuilder {
    fn get_default_visibility(&self) -> Visibility {
        self.builder.borrow().property_visibility
    }

    fn add_property(&mut self, property: Property) {
        self.properties.push(property)
    }
}