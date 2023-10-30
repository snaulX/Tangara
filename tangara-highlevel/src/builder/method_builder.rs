use crate::builder::generate_member_id;
use crate::{Argument, ArgumentKind, Method, TypeRef, Value, Visibility};

pub trait MethodCollector {
    fn get_default_visibility(&self) -> Visibility;
    fn add_method(&mut self, method: Method);
}

pub struct MethodBuilder<'a, T: MethodCollector> {
    builder: &'a mut T,
    vis: Visibility,
    name: String,
    args: Vec<Argument>,
    return_type: Option<TypeRef>
}

impl<'a, T: MethodCollector> MethodBuilder<'a, T> {
    pub(crate) fn new(builder: &'a mut T, name: &str) -> Self {
        let vis = builder.get_default_visibility();
        Self {
            builder,
            vis,
            name: name.to_string(),
            args: Vec::new(),
            return_type: None
        }
    }

    pub fn set_visibility(&mut self, vis: Visibility) -> &mut Self {
        self.vis = vis;
        self
    }

    /// Set return type. If you don't (by not calling this method) it will be `void` - nothing.
    /// But if you will, you cannot return it back to nothing.
    pub fn return_type(&mut self, return_type: TypeRef) -> &mut Self {
        self.return_type = Some(return_type);
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

    /// Creates output argument with given type and name
    pub fn arg_out(&mut self, arg_type: TypeRef, name: &str) -> &mut Self {
        self.args.push(Argument(arg_type, name.to_string(), ArgumentKind::Out));
        self
    }

    /// Creates reference argument with given type and name
    pub fn arg_ref(&mut self, arg_type: TypeRef, name: &str) -> &mut Self {
        self.args.push(Argument(arg_type, name.to_string(), ArgumentKind::Ref));
        self
    }

    pub fn get_method(&self) -> Method {
        Method {
            vis: self.vis,
            name: self.name.clone(),
            id: generate_member_id(&self.name),
            args: self.args.to_vec(),
            return_type: self.return_type.clone(),
        }
    }
    
    pub fn build(&'a mut self) -> &'a mut T {
        self.builder.add_method(self.get_method());
        self.builder
    }
}