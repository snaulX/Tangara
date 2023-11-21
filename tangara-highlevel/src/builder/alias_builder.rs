use std::cell::RefCell;
use std::rc::Rc;
use crate::builder::{generate_type_id, GenericsCollector, PackageBuilder, TypeBuilder};
use crate::{Attribute, Generics, Type, TypeRef, Visibility};
use crate::TypeKind::TypeAlias;

pub struct TypeAliasBuilder {
    builder: Rc<RefCell<PackageBuilder>>,
    attrs: Vec<Attribute>,
    name: String,
    namespace: String,
    vis: Visibility,
    alias: TypeRef,
    generics: Vec<String>,
    generics_where: Vec<(String, TypeRef)>
}

impl TypeAliasBuilder {
    pub fn new(builder: Rc<RefCell<PackageBuilder>>, name: &str, alias: TypeRef) -> Self {
        let namespace = builder.borrow().get_namespace();
        let vis = builder.borrow().type_visibility;
        Self {
            builder,
            attrs: vec![],
            name: name.to_string(),
            namespace,
            vis,
            alias,
            generics: vec![],
            generics_where: vec![]
        }
    }
}

impl TypeBuilder for TypeAliasBuilder {
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

impl GenericsCollector for TypeAliasBuilder {
    fn generic(&mut self, generic: String) -> &mut Self {
        self.generics.push(generic);
        self
    }

    /// Add statement for generics `where statement.0: statement.1`.
    /// Function *panics* if first type doesn't exists in generics of this type alias.
    fn generic_where(&mut self, generic_where: (String, TypeRef)) -> &mut Self {
        if !self.generics.contains(&generic_where.0) {
            panic!(
                "Generic {} doesn't exists in this type alias, so it can't be used in 'where' statement",
                generic_where.0);
        }
        self.generics_where.push(generic_where);
        self
    }
}