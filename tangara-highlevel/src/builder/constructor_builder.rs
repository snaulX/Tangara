use crate::{Argument, ArgumentKind, Attribute, Constructor, TypeRef, Value, Visibility};

pub trait ConstructorCollector {
    fn get_default_visibility(&self) -> Visibility;
    fn add_constructor(&mut self, constructor: Constructor);
}

pub struct ConstructorBuilder<'a, T: ConstructorCollector> {
    builder: &'a mut T,
    attrs: Vec<Attribute>,
    vis: Visibility,
    arg_attrs: Vec<Attribute>,
    args: Vec<Argument>
}

impl<'a, T: ConstructorCollector> ConstructorBuilder<'a, T> {
    pub(crate) fn new(builder: &'a mut T) -> Self {
        let vis = builder.get_default_visibility();
        Self {
            builder,
            attrs: vec![],
            vis,
            arg_attrs: Vec::new(),
            args: Vec::new()
        }
    }

    pub fn add_attribute(&mut self, attr: Attribute) -> &mut Self {
        self.attrs.push(attr);
        self
    }

    pub fn set_visibility(&mut self, vis: Visibility) -> &mut Self {
        self.vis = vis;
        self
    }

    /// Push attribute before next argument
    pub fn arg_attribute(&mut self, attribute: Attribute) -> &mut Self {
        self.arg_attrs.push(attribute);
        self
    }

    #[inline]
    fn add_argument(&mut self, arg_type: TypeRef, name: &str, kind: ArgumentKind) -> &mut Self {
        self.args.push(Argument(
            self.arg_attrs.to_vec(),
            arg_type,
            name.to_string(),
            kind
        ));
        self.arg_attrs.clear();
        self
    }

    /// Creates common argument with given type and name
    pub fn arg(&mut self, arg_type: TypeRef, name: &str) -> &mut Self {
        self.add_argument(arg_type, name, ArgumentKind::Default)
    }

    /// Creates common argument with given type, name and default value
    pub fn arg_value(&mut self, arg_type: TypeRef, name: &str, default_value: Value) -> &mut Self {
        self.add_argument(arg_type, name, ArgumentKind::DefaultValue(default_value))
    }

    /*
    Does we really need Out argument in constructor?

    /// Creates output argument with given type and name
    pub fn arg_out(&mut self, arg_type: TypeRef, name: &str) -> &mut Self {
        self.add_argument(arg_type, name, ArgumentKind::Out)
    }
     */

    /// Creates reference argument with given type and name
    pub fn arg_ref(&mut self, arg_type: TypeRef, name: &str) -> &mut Self {
        self.add_argument(arg_type, name, ArgumentKind::Ref)
    }

    pub fn get_constructor(&self) -> Constructor {
        Constructor {
            attrs: self.attrs.to_vec(),
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