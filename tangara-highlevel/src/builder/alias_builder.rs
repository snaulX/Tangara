use crate::builder::{generate_type_id, PackageBuilder, TypeBuilder};
use crate::{Type, TypeRef, Visibility};
use crate::TypeKind::TypeAlias;

pub struct TypeAliasBuilder<'a> {
    builder: &'a mut PackageBuilder,
    name: String,
    vis: Visibility,
    alias: TypeRef
}

impl<'a> TypeAliasBuilder<'a> {
    pub(crate) fn new(builder: &'a mut PackageBuilder, name: &str, alias: TypeRef) -> Self {
        let vis = builder.type_visibility.clone();
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

impl<'a> TypeBuilder for TypeAliasBuilder<'a> {
    fn get_type(&self) -> Type {
        Type {
            vis: self.vis.clone(),
            namespace: self.builder.namespace.clone(),
            name: self.name.clone(),
            id: generate_type_id(&self.name),
            attrs: vec![],
            kind: TypeAlias(Box::new(self.alias.clone()))
        }
    }

    fn build(&mut self) -> &mut PackageBuilder {
        self.builder.types.push(self.get_type());
        self.builder
    }
}