use std::cell::RefCell;
use std::rc::Rc;
use crate::builder::{generate_type_id, GenericsCollector, PackageBuilder, TypeBuilder};
use crate::{Attribute, Constructor, Generics, Method, Property, Type, TypeRef, Visibility};
use crate::builder::constructor_builder::{ConstructorBuilder, ConstructorCollector};
use crate::builder::method_builder::{MethodBuilder, MethodCollector};
use crate::builder::property_builder::{PropertyBuilder, PropertyCollector};
use crate::TypeKind::Class;

pub struct ClassBuilder {
    builder: Rc<RefCell<PackageBuilder>>,
    attrs: Vec<Attribute>,
    name: String,
    vis: Visibility,
    constructors: Vec<Constructor>,
    properties: Vec<Property>,
    methods: Vec<Method>,
    parents: Vec<TypeRef>,
    generics: Vec<String>,
    generics_where: Vec<(String, TypeRef)>,
    sealed: bool
}

impl ClassBuilder {
    pub fn new(builder: Rc<RefCell<PackageBuilder>>, name: &str) -> Self {
        let vis = builder.borrow().type_visibility;
        Self {
            builder,
            attrs: vec![],
            name: name.to_string(),
            vis,
            constructors: vec![],
            properties: vec![],
            methods: vec![],
            parents: vec![],
            generics: vec![],
            generics_where: vec![],
            sealed: true
        }
    }

    pub fn inherits(&mut self, parent: TypeRef) -> &mut Self {
        self.parents.push(parent);
        self
    }

    pub fn set_visibility(&mut self, vis: Visibility) -> &mut Self {
        self.vis = vis;
        self
    }

    /// Make class open to inherit from
    pub fn open(&mut self) -> &mut Self {
        self.sealed = false;
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
    fn add_attribute(&mut self, attr: Attribute) -> &mut Self {
        self.attrs.push(attr);
        self
    }

    fn get_type(&self) -> Type {
        let namespace = self.builder.borrow().namespace.clone();
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
            kind: Class(
                self.sealed,
                self.constructors.to_vec(),
                self.properties.to_vec(),
                self.methods.to_vec(),
                self.parents.to_vec(),
            )
        }
    }

    fn build(self) -> Type {
        let result_type = self.get_type();
        let mut builder = self.builder.borrow_mut();
        builder.add_type(result_type.clone());
        result_type
    }
}

impl GenericsCollector for ClassBuilder {
    fn generic(&mut self, generic: String) -> &mut Self {
        self.generics.push(generic);
        self
    }

    /// Add statement for generics `where statement.0: statement.1`.
    /// Function *panics* if first type doesn't exists in generics of this class.
    fn generic_where(&mut self, statement: (String, TypeRef)) -> &mut Self {
        if !self.generics.contains(&statement.0) {
            panic!(
                "Generic {} doesn't exists in this class, so it can't be used in 'where' statement",
                statement.0);
        }
        self.generics_where.push(statement);
        self
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