use std::cell::RefCell;
use std::rc::Rc;
use crate::builder::{GenericsCollector, PackageBuilder, TypeBuilder};
use crate::{Attribute, Constructor, generate_type_id, Generics, Property, Type, TypeRef, Visibility};
use crate::builder::constructor_builder::*;
use crate::builder::property_builder::*;
use crate::TypeKind::Struct;

pub struct StructBuilder {
    builder: Rc<RefCell<PackageBuilder>>,
    attrs: Vec<Attribute>,
    name: String,
    namespace: String,
    vis: Visibility,
    constructors: Vec<Constructor>,
    properties: Vec<Property>,
    generics: Vec<String>,
    generics_where: Vec<(String, TypeRef)>
}

impl StructBuilder {
    pub(crate) fn new(builder: Rc<RefCell<PackageBuilder>>, name: &str) -> Self {
        let namespace = builder.borrow().get_namespace();
        let vis = builder.borrow().type_visibility;
        Self {
            builder,
            attrs: vec![],
            name: name.to_string(),
            namespace,
            vis,
            constructors: Vec::new(),
            properties: Vec::new(),
            generics: vec![],
            generics_where: vec![]
        }
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

    fn set_visibility(&mut self, vis: Visibility) -> &mut Self {
        self.vis = vis;
        self
    }

    fn get_type(&self) -> Type {
        let namespace = self.namespace.clone();
        let name = self.name.clone();
        let mut full_name = String::with_capacity(namespace.len() + name.len() + 1);
        full_name.push_str(&namespace);
        full_name.push('.');
        full_name.push_str(&name);
        let id = generate_type_id(&full_name);
        Type {
            attrs: self.attrs.to_vec(),
            vis: self.vis.clone(),
            namespace,
            name,
            id,
            generics: Generics(self.generics.to_vec(), self.generics_where.to_vec()),
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

impl GenericsCollector for StructBuilder {
    fn generic(&mut self, generic: String) -> &mut Self {
        self.generics.push(generic);
        self
    }

    /// Add statement for generics `where statement.0: statement.1`.
    /// Function *panics* if first type doesn't exists in generics of this struct.
    fn generic_where(&mut self, statement: (String, TypeRef)) -> &mut Self {
        if !self.generics.contains(&statement.0) {
            panic!(
                "Generic {} doesn't exists in this struct, so it can't be used in 'where' statement",
                statement.0);
        }
        self.generics_where.push(statement);
        self
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