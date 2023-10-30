use crate::builder::{ConstructorCollector, generate_typeid, PackageBuilder, PropertyCollector, TypeBuilder};
use crate::{Constructor, Property, Type, TypeRef, Visibility};
use crate::builder::constructor_builder::ConstructorBuilder;
use crate::builder::property_builder::PropertyBuilder;
use crate::TypeKind::Class;

pub struct ClassBuilder<'a> {
    builder: &'a mut PackageBuilder,
    name: String,
    vis: Visibility,
    constructors: Vec<Constructor>,
    properties: Vec<Property>
}

impl<'a> ClassBuilder<'a> {
    pub(crate) fn new(builder: &'a mut PackageBuilder, name: &str) -> Self {
        let vis = builder.type_visibility.clone();
        Self {
            builder,
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

    pub fn add_constructor(&'a mut self) -> ConstructorBuilder<'a, Self> {
        ConstructorBuilder::new(self, self.builder.constructor_visibility)
    }

    pub fn add_property(&'a mut self, prop_type: TypeRef, name: &str) -> PropertyBuilder<'a, Self> {
        PropertyBuilder::new(self, prop_type, name)
    }
}

impl<'a> TypeBuilder for ClassBuilder<'a> {
    fn get_type(&self) -> Type {
        Type {
            vis: self.vis.clone(),
            namespace: self.builder.namespace.clone(),
            name: self.name.clone(),
            id: generate_typeid(&self.name),
            attrs: vec![],
            kind: Class(self.constructors.to_vec(), self.properties.to_vec())
        }
    }

    fn build(&mut self) -> &mut PackageBuilder {
        self.builder.types.push(self.get_type());
        self.builder
    }
}

impl<'a> ConstructorCollector for ClassBuilder<'a> {
    fn get_default_visibility(&self) -> Visibility {
        self.builder.constructor_visibility
    }

    fn add_constructor(&mut self, constructor: Constructor) {
        self.constructors.push(constructor)
    }
}

impl<'a> PropertyCollector for ClassBuilder<'a> {
    fn get_default_visibility(&self) -> Visibility {
        self.builder.property_visibility
    }

    fn add_property(&mut self, property: Property) {
        self.properties.push(property)
    }
}