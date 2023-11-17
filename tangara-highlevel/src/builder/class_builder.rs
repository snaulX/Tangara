use std::cell::RefCell;
use std::rc::Rc;
use crate::builder::{generate_type_id, PackageBuilder, TypeBuilder};
use crate::{Constructor, Method, Property, Type, TypeRef, Visibility};
use crate::builder::constructor_builder::{ConstructorBuilder, ConstructorCollector};
use crate::builder::method_builder::{MethodBuilder, MethodCollector};
use crate::builder::property_builder::{PropertyBuilder, PropertyCollector};
use crate::TypeKind::Class;

pub struct ClassBuilder {
    builder: Rc<RefCell<PackageBuilder>>,
    name: String,
    vis: Visibility,
    constructors: Vec<Constructor>,
    properties: Vec<Property>,
    methods: Vec<Method>
}

impl ClassBuilder {
    pub fn new(builder: Rc<RefCell<PackageBuilder>>, name: &str) -> Self {
        let vis = builder.borrow().type_visibility;
        Self {
            builder,
            name: name.to_string(),
            vis,
            constructors: Vec::new(),
            properties: Vec::new(),
            methods: Vec::new()
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

    pub fn add_method(&mut self, name: &str) -> MethodBuilder<Self> {
        MethodBuilder::new(self, name)
    }
}

impl TypeBuilder for ClassBuilder {
    fn get_type(&self) -> Type {
        Type {
            vis: self.vis.clone(),
            namespace: self.builder.borrow().namespace.clone(),
            name: self.name.clone(),
            id: generate_type_id(&self.name),
            attrs: vec![],
            kind: Class(
                self.constructors.to_vec(),
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

impl ConstructorCollector for ClassBuilder {
    fn get_default_visibility(&self) -> Visibility {
        self.builder.borrow().constructor_visibility
    }

    fn add_constructor(&mut self, constructor: Constructor) {
        self.constructors.push(constructor)
    }
}

impl PropertyCollector for ClassBuilder {
    fn get_default_visibility(&self) -> Visibility {
        self.builder.borrow().property_visibility
    }

    fn add_property(&mut self, property: Property) {
        self.properties.push(property)
    }
}

impl MethodCollector for ClassBuilder {
    fn get_default_visibility(&self) -> Visibility {
        self.builder.borrow().method_visibility
    }

    fn add_method(&mut self, method: Method) {
        self.methods.push(method)
    }
}