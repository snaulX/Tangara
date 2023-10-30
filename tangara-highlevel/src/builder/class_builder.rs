use crate::builder::{generate_typeid, PackageBuilder, TypeBuilder};
use crate::{Argument, Constructor, Property, Type, TypeRef, Visibility};
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

    pub fn add_constructor(&'a mut self) -> ConstructorBuilder<'a> {
        ConstructorBuilder::new(self, self.builder.constructor_visibility)
    }

    pub fn add_property(&'a mut self, prop_type: TypeRef, name: &str) -> PropertyBuilder<'a> {
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

pub struct PropertyBuilder<'a> {
    builder: &'a mut ClassBuilder<'a>,
    getter_visibility: Visibility,
    setter_visibility: Option<Visibility>,
    prop_type: TypeRef,
    name: String
}

impl<'a> PropertyBuilder<'a> {
    pub(crate) fn new(builder: &'a mut ClassBuilder<'a>, prop_type: TypeRef, name: &str) -> Self {
        Self {
            builder,
            getter_visibility: Visibility::Public/*builder.builder.property_visibility*/,
            setter_visibility: None,
            prop_type,
            name: name.to_string()
        }
    }

    pub fn getter_visibility(&mut self, vis: Visibility) -> &mut Self {
        self.getter_visibility = vis;
        self
    }

    pub fn setter_visibility(&mut self, vis: Visibility) -> &mut Self {
        self.setter_visibility = Some(vis);
        self
    }

    pub fn get_prop(&self) -> Property {
        Property {
            getter_visibility: self.getter_visibility,
            setter_visibility: self.setter_visibility,
            prop_type: self.prop_type.clone(),
            name: self.name.clone(),
        }
    }

    pub fn build(&'a mut self) -> &'a mut ClassBuilder<'a> {
        self.builder.properties.push(self.get_prop());
        self.builder
    }
}

pub struct ConstructorBuilder<'a> {
    builder: &'a mut ClassBuilder<'a>,
    vis: Visibility,
    args: Vec<Argument>
}

impl<'a> ConstructorBuilder<'a> {
    pub(crate) fn new(builder: &'a mut ClassBuilder<'a>, vis: Visibility) -> Self {
        Self {
            builder,
            vis,
            args: Vec::new()
        }
    }

    pub fn set_visibility(&mut self, vis: Visibility) -> &mut Self {
        self.vis = vis;
        self
    }

    pub fn arg(&mut self, arg_type: TypeRef, name: &str) -> &mut Self {
        self.args.push(Argument(arg_type, name.to_string(), None));
        self
    }

    pub fn get_ctor(&self) -> Constructor {
        Constructor {
            vis: self.vis,
            args: self.args.to_vec(),
        }
    }

    pub fn build(&'a mut self) -> &'a mut ClassBuilder<'a> {
        self.builder.constructors.push(self.get_ctor());
        self.builder
    }
}