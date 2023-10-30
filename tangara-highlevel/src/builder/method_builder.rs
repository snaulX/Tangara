use crate::builder::generate_member_id;
use crate::{Method, TypeRef, Visibility};

pub trait MethodCollector {
    fn get_default_visibility(&self) -> Visibility;
    fn add_method(&mut self, method: Method);
}

pub struct MethodBuilder<'a, T: MethodCollector> {
    builder: &'a mut T,
    vis: Visibility,
    name: String,
    return_type: Option<TypeRef>
}

impl<'a, T: MethodCollector> MethodBuilder<'a, T> {
    pub(crate) fn new(builder: &'a mut T, name: &str) -> Self {
        let vis = builder.get_default_visibility();
        Self {
            builder,
            vis,
            name: name.to_string(),
            return_type: None
        }
    }

    pub fn set_visibility(&mut self, vis: Visibility) -> &mut Self {
        self.vis = vis;
        self
    }

    pub fn return_type(&mut self, return_type: TypeRef) -> &mut Self {
        self.return_type = Some(return_type);
        self
    }

    pub fn get_method(&self) -> Method {
        Method {
            vis: self.vis,
            name: self.name.clone(),
            id: generate_member_id(&self.name),
            args: vec![],
            return_type: self.return_type.clone(),
        }
    }
    
    pub fn build(&'a mut self) -> &'a mut T {
        self.builder.add_method(self.get_method());
        self.builder
    }
}