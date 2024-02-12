use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use crate::builder::{GenericsCollector, PackageBuilder, TypeBuilder};
use crate::{Attribute, generate_type_id, Generics, Method, MethodKind, Property, Type, TypeRef, Visibility};
use crate::builder::method_builder::{MethodBuilder, MethodCollector};
use crate::builder::property_builder::{PropertyBuilder, PropertyCollector};
use crate::TypeKind::Interface;

pub struct InterfaceBuilder {
    builder: Rc<RefCell<PackageBuilder>>,
    attrs: Vec<Attribute>,
    name: String,
    namespace: String,
    vis: Visibility,
    properties: Vec<Property>,
    methods: Vec<Method>,
    parents: Vec<TypeRef>,
    generics: Vec<String>,
    generics_where: Vec<(String, TypeRef)>
}

impl InterfaceBuilder {
    pub fn new(builder: Rc<RefCell<PackageBuilder>>, name: &str) -> Self {
        let namespace = builder.borrow().get_namespace();
        let vis = builder.borrow().type_visibility;
        Self {
            builder,
            attrs: vec![],
            name: name.to_string(),
            namespace,
            vis,
            properties: vec![],
            methods: vec![],
            parents: vec![],
            generics: vec![],
            generics_where: vec![]
        }
    }

    pub fn inherits(&mut self, parent: TypeRef) -> &mut Self {
        self.parents.push(parent);
        self
    }

    pub fn add_property(&mut self, prop_type: TypeRef, name: &str) -> PropertyBuilder<Self> {
        PropertyBuilder::new(self, prop_type, name)
    }

    pub fn add_method(&mut self, name: &str) -> MethodBuilder<Self> {
        let mut builder = MethodBuilder::new(self, name);
        builder.set_kind(MethodKind::Abstract);
        builder
    }
}

impl TypeBuilder for InterfaceBuilder {
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
            kind: Interface {
                properties: self.properties.to_vec(),
                methods: self.methods.to_vec(),
                parents: self.parents.to_vec()
            }
        }
    }

    fn build(self) -> Type {
        let result_type = self.get_type();
        let mut builder = self.builder.borrow_mut();
        builder.types.push(result_type.clone());
        result_type
    }
}

impl GenericsCollector for InterfaceBuilder {
    fn generic(&mut self, generic: String) -> &mut Self {
        self.generics.push(generic);
        self
    }

    /// Add statement for generics `where statement.0: statement.1`.
    /// Function *panics* if first type doesn't exists in generics of this interface.
    fn generic_where(&mut self, statement: (String, TypeRef)) -> &mut Self {
        if !self.generics.contains(&statement.0) {
            panic!(
                "Generic {} doesn't exists in this interface, so it can't be used in 'where' statement",
                statement.0);
        }
        self.generics_where.push(statement);
        self
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

    fn get_supported_kinds(&self) -> HashSet<MethodKind> {
        HashSet::from([MethodKind::Abstract])
    }

    fn add_method(&mut self, method: Method) {
        if method.vis == Visibility::Private {
            panic!("Visibility of interface member cannot be private")
        }
        self.methods.push(method)
    }
}