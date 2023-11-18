use std::cell::RefCell;
use std::rc::Rc;
use crate::builder::{generate_type_id, PackageBuilder, TypeBuilder};
use crate::{Attribute, Method, Property, Type, TypeRef, Visibility};
use crate::builder::method_builder::{MethodBuilder, MethodCollector};
use crate::builder::property_builder::{PropertyBuilder, PropertyCollector};
use crate::TypeKind::Interface;

pub struct InterfaceBuilder {
    builder: Rc<RefCell<PackageBuilder>>,
    attrs: Vec<Attribute>,
    name: String,
    vis: Visibility,
    properties: Vec<Property>,
    methods: Vec<Method>
}

impl InterfaceBuilder {
    pub fn new(builder: Rc<RefCell<PackageBuilder>>, name: &str) -> Self {
        let vis = builder.borrow().type_visibility;
        Self {
            builder,
            attrs: vec![],
            name: name.to_string(),
            vis,
            properties: Vec::new(),
            methods: Vec::new()
        }
    }

    pub fn set_visibility(&mut self, vis: Visibility) -> &mut Self {
        self.vis = vis;
        self
    }

    pub fn add_property(&mut self, prop_type: TypeRef, name: &str) -> PropertyBuilder<Self> {
        PropertyBuilder::new(self, prop_type, name)
    }

    pub fn add_method(&mut self, name: &str) -> MethodBuilder<Self> {
        MethodBuilder::new(self, name)
    }
}

impl TypeBuilder for InterfaceBuilder {
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
            kind: Interface(
                self.properties.to_vec(),
                self.methods.to_vec()
            )
        }
    }

    fn build(self) -> Type {
        let result_type = self.get_type();
        let mut builder = self.builder.borrow_mut();
        builder.types.push(result_type.clone());
        result_type
    }
}

impl PropertyCollector for InterfaceBuilder {
    fn get_default_visibility(&self) -> Visibility {
        Visibility::Public
    }

    fn add_property(&mut self, property: Property) {
        if property.getter_visibility == Visibility::Private &&
            property.setter_visibility.unwrap_or(Visibility::Private) == Visibility::Private {
            panic!("Visibility of interface member cannot be private")
        }
        self.properties.push(property)
    }
}

impl MethodCollector for InterfaceBuilder {
    fn get_default_visibility(&self) -> Visibility {
        Visibility::Public
    }

    fn add_method(&mut self, method: Method) {
        if method.vis == Visibility::Private {
            panic!("Visibility of interface member cannot be private")
        }
        self.methods.push(method)
    }
}