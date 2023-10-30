use crate::builder::PropertyCollector;
use crate::{Property, TypeRef, Visibility};

pub struct PropertyBuilder<'a, T: PropertyCollector> {
    builder: &'a mut T,
    getter_visibility: Visibility,
    setter_visibility: Option<Visibility>,
    prop_type: TypeRef,
    name: String
}

impl<'a, T: PropertyCollector> PropertyBuilder<'a, T> {
    pub(crate) fn new(builder: &'a mut T, prop_type: TypeRef, name: &str) -> Self {
        let getter_visibility = builder.get_default_visibility();
        Self {
            builder,
            getter_visibility,
            setter_visibility: None,
            prop_type,
            name: name.to_string()
        }
    }

    pub fn getter_visibility(&mut self, vis: Visibility) -> &mut Self {
        self.getter_visibility = vis;
        self
    }

    pub fn setter_visibility(&mut self, vis: Visibility) -> &mut Self {
        self.setter_visibility = Some(vis);
        self
    }

    pub fn get_property(&self) -> Property {
        Property {
            getter_visibility: self.getter_visibility,
            setter_visibility: self.setter_visibility,
            prop_type: self.prop_type.clone(),
            name: self.name.clone(),
        }
    }

    pub fn build(&'a mut self) -> &'a mut T {
        self.builder.add_property(self.get_property());
        self.builder
    }
}