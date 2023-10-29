use crate::builder::{generate_typeid, PackageBuilder};
use crate::{Type, Visibility};
use crate::TypeKind::Class;

pub struct ClassBuilder<'a> {
    builder: &'a mut PackageBuilder,
    name: String,
    vis: Visibility
}

impl<'a> ClassBuilder<'a> {
    pub(crate) fn new(builder: &'a mut PackageBuilder, name: &str) -> Self {
        let vis = builder.type_visibility.clone();
        Self {
            builder,
            name: name.to_string(),
            vis
        }
    }

    pub fn set_visibility(&mut self, vis: Visibility) -> &mut Self {
        self.vis = vis;
        self
    }

    pub fn build(&'a mut self) -> &'a mut PackageBuilder {
        self.builder.types.push(Type {
            vis: self.vis.clone(),
            namespace: self.builder.namespace.clone(),
            name: self.name.clone(),
            id: generate_typeid(&self.name),
            attrs: vec![],
            kind: Class()
        });
        self.builder
    }
}