use crate::builder::AttributeCollector;
use crate::{Attribute, generate_property_id, Property, TypeRef, Visibility};

pub trait PropertyCollector {
    fn get_default_visibility(&self) -> Visibility;
    fn add_property(&mut self, property: Property);
}

pub struct PropertyBuilder<'a, T: PropertyCollector> {
    builder: &'a mut T,
    attrs: Vec<Attribute>,
    getter_visibility: Visibility,
    setter_visibility: Option<Visibility>,
    prop_type: TypeRef,
    name: String
}

impl<'a, T: PropertyCollector> PropertyBuilder<'a, T> {
    pub(crate) fn new(builder: &'a mut T, prop_type: TypeRef, name: &str) -> Self {
        let getter_visibility = builder.get_default_visibility();
        Self {
            attrs: vec![],
            builder,
            getter_visibility,
            setter_visibility: None,
            prop_type,
            name: name.to_string()
        }
    }

    /// Set visibility for getter
    pub fn getter_visibility(&mut self, vis: Visibility) -> &mut Self {
        self.getter_visibility = vis;
        self
    }

    /// Set visibility for setter. If setter wasn't exists - it creates it.
    pub fn setter_visibility(&mut self, vis: Visibility) -> &mut Self {
        self.setter_visibility = Some(vis);
        self
    }

    pub fn get_property(&self) -> Property {
        Property {
            attrs: self.attrs.to_vec(),
            getter_visibility: self.getter_visibility,
            setter_visibility: self.setter_visibility,
            prop_type: self.prop_type.clone(),
            name: self.name.clone(),
            id: generate_property_id(&self.name)
        }
    }

    /// Pass property to parent builder and returns it
    pub fn build(&'a mut self) -> &'a mut T {
        self.builder.add_property(self.get_property());
        self.builder
    }
}

impl<T: PropertyCollector> AttributeCollector for PropertyBuilder<'_, T> {
    fn add_attribute(&mut self, attribute: Attribute) -> &mut Self {
        self.attrs.push(attribute);
        self
    }
}