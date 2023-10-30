use crate::builder::ConstructorCollector;
use crate::{Argument, Constructor, TypeRef, Visibility};

pub struct ConstructorBuilder<'a, T: ConstructorCollector> {
    builder: &'a mut T,
    vis: Visibility,
    args: Vec<Argument>
}

impl<'a, T: ConstructorCollector> ConstructorBuilder<'a, T> {
    pub(crate) fn new(builder: &'a mut T, vis: Visibility) -> Self {
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

    pub fn get_constructor(&self) -> Constructor {
        Constructor {
            vis: self.vis,
            args: self.args.to_vec(),
        }
    }

    pub fn build(&'a mut self) -> &'a mut T {
        self.builder.add_constructor(self.get_constructor());
        self.builder
    }
}