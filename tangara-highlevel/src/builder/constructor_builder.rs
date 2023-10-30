use crate::{Argument, ArgumentKind, Constructor, TypeRef, Value, Visibility};

pub trait ConstructorCollector {
    fn get_default_visibility(&self) -> Visibility;
    fn add_constructor(&mut self, constructor: Constructor);
}

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

    /// Creates common argument with given type and name
    pub fn arg(&mut self, arg_type: TypeRef, name: &str) -> &mut Self {
        self.args.push(Argument(arg_type, name.to_string(), ArgumentKind::Default));
        self
    }

    /// Creates common argument with given type, name and default value
    pub fn arg_value(&mut self, arg_type: TypeRef, name: &str, default_value: Value) -> &mut Self {
        self.args.push(Argument(arg_type, name.to_string(), ArgumentKind::DefaultValue(default_value)));
        self
    }

    /// Creates reference argument with given type and name
    pub fn arg_ref(&mut self, arg_type: TypeRef, name: &str) -> &mut Self {
        self.args.push(Argument(arg_type, name.to_string(), ArgumentKind::Ref));
        self
    }

    // Does we need 'out' argument in constructor?

    pub fn get_constructor(&self) -> Constructor {
        Constructor {
            vis: self.vis,
            args: self.args.to_vec(),
        }
    }

    /// Pass constructor to parent builder and returns it
    pub fn build(&'a mut self) -> &'a mut T {
        self.builder.add_constructor(self.get_constructor());
        self.builder
    }
}