use crate::builder::{generate_type_id, PackageBuilder, TypeBuilder};
use crate::{Constructor, Method, Property, Type, TypeRef, Visibility};
use crate::builder::constructor_builder::{ConstructorBuilder, ConstructorCollector};
use crate::builder::method_builder::{MethodBuilder, MethodCollector};
use crate::builder::property_builder::{PropertyBuilder, PropertyCollector};
use crate::TypeKind::{Class, Interface};

pub struct InterfaceBuilder<'a> {
    builder: &'a mut PackageBuilder,
    name: String,
    vis: Visibility,
    properties: Vec<Property>,
    methods: Vec<Method>
}

impl<'a> InterfaceBuilder<'a> {
    pub(crate) fn new(builder: &'a mut PackageBuilder, name: &str) -> Self {
        let vis = builder.type_visibility.clone();
        Self {
            builder,
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

    pub fn add_property(&'a mut self, prop_type: TypeRef, name: &str) -> PropertyBuilder<'a, Self> {
        PropertyBuilder::new(self, prop_type, name)
    }

    pub fn add_method(&'a mut self, name: &str) -> MethodBuilder<'a, Self> {
        MethodBuilder::new(self, name)
    }
}

impl<'a> TypeBuilder for InterfaceBuilder<'a> {
    fn get_type(&self) -> Type {
        Type {
            vis: self.vis.clone(),
            namespace: self.builder.namespace.clone(),
            name: self.name.clone(),
            id: generate_type_id(&self.name),
            attrs: vec![],
            kind: Interface(
                self.properties.to_vec(),
                self.methods.to_vec()
            )
        }
    }

    fn build(&mut self) -> &mut PackageBuilder {
        self.builder.types.push(self.get_type());
        self.builder
    }
}

impl<'a> PropertyCollector for InterfaceBuilder<'a> {
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

impl<'a> MethodCollector for InterfaceBuilder<'a> {
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