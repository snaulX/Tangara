use std::cell::RefCell;
use std::rc::Rc;
use crate::builder::{generate_type_id, PackageBuilder, TypeBuilder};
use crate::{Type, TypeRef, Visibility};
use crate::TypeKind::TypeAlias;

pub struct TypeAliasBuilder {
    builder: Rc<RefCell<PackageBuilder>>,
    name: String,
    vis: Visibility,
    alias: TypeRef
}

impl TypeAliasBuilder {
    pub fn new(builder: Rc<RefCell<PackageBuilder>>, name: &str, alias: TypeRef) -> Self {
        let vis = builder.borrow().type_visibility.clone();
        Self {
            builder,
            name: name.to_string(),
            vis,
            alias
        }
    }

    pub fn set_visibility(&mut self, vis: Visibility) -> &mut Self {
        self.vis = vis;
        self
    }
}

impl TypeBuilder for TypeAliasBuilder {
    fn get_type(&self) -> Type {
        Type {
            vis: self.vis.clone(),
            namespace: self.builder.borrow().namespace.clone(),
            name: self.name.clone(),
            id: generate_type_id(&self.name),
            attrs: vec![],
            kind: TypeAlias(Box::new(self.alias.clone()))
        }
    }

    fn build(self) -> Type {
        let result_type = self.get_type();
        let mut builder = self.builder.borrow_mut();
        builder.types.push(result_type.clone());
        result_type
    }
}