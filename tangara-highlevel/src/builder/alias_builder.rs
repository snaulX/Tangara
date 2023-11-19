use std::cell::RefCell;
use std::rc::Rc;
use crate::builder::{generate_type_id, PackageBuilder, TypeBuilder};
use crate::{Attribute, Generics, Type, TypeRef, Visibility};
use crate::TypeKind::TypeAlias;

pub struct TypeAliasBuilder {
    builder: Rc<RefCell<PackageBuilder>>,
    attrs: Vec<Attribute>,
    name: String,
    vis: Visibility,
    alias: TypeRef,
    generics: Vec<String>,
    generics_where: Vec<(String, TypeRef)>
}

impl TypeAliasBuilder {
    pub fn new(builder: Rc<RefCell<PackageBuilder>>, name: &str, alias: TypeRef) -> Self {
        let vis = builder.borrow().type_visibility.clone();
        Self {
            builder,
            attrs: vec![],
            name: name.to_string(),
            vis,
            alias,
            generics: vec![],
            generics_where: vec![]
        }
    }

    pub fn set_visibility(&mut self, vis: Visibility) -> &mut Self {
        self.vis = vis;
        self
    }

    /// Set generic types for this type alias.
    /// If generics already exists - **it rewrites old**.
    pub fn generics(&mut self, generics: Vec<String>) -> &mut Self {
        self.generics = generics;
        self
    }

    /// Add statement for generics `where statement.0: statement.1`.
    /// Function *panics* if first type doesn't exists in generics of this type alias.
    pub fn generic_where(&mut self, statement: (String, TypeRef)) -> &mut Self {
        if !self.generics.contains(&statement.0) {
            panic!(
                "Generic {} doesn't exists in this type alias, so it can't be used in 'where' statement",
                statement.0);
        }
        self.generics_where.push(statement);
        self
    }
}

impl TypeBuilder for TypeAliasBuilder {
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
            kind: TypeAlias(Box::new(self.alias.clone()))
        }
    }

    fn build(self) -> Type {
        let result_type = self.get_type();
        let mut builder = self.builder.borrow_mut();
        builder.add_type(result_type.clone());
        result_type
    }
}