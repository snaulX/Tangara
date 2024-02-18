use crate::builder::AttributeCollector;
use crate::{Attribute, Field, generate_member_id, Property, TypeRef, Value, Visibility};

pub trait FieldCollector {
    fn get_default_visibility(&self) -> Visibility;
    fn add_field(&mut self, field: Field);
    fn add_static_field(&mut self, field: Field);
}

pub struct FieldBuilder<'a, T: FieldCollector> {
    builder: &'a mut T,
    is_static: bool,
    attrs: Vec<Attribute>,
    visibility: Visibility,
    field_type: TypeRef,
    default_value: Option<Value>,
    name: String
}

impl<'a, T: FieldCollector> FieldBuilder<'a, T> {
    pub(crate) fn new(builder: &'a mut T, is_static: bool, field_type: TypeRef, name: &str) -> Self {
        let visibility = builder.get_default_visibility();
        Self {
            attrs: vec![],
            builder,
            is_static,
            visibility,
            field_type,
            default_value: None,
            name: name.to_string()
        }
    }

    pub fn set_visibility(&mut self, vis: Visibility) -> &mut Self {
        self.visibility = vis;
        self
    }

    /// Note: Don't call this method if you don't want to set the default value
    pub fn set_default_value(&mut self, value: Value) -> &mut Self {
        self.default_value = Some(value);
        self
    }

    pub fn get_field(&self) -> Field {
        Field {
            attrs: self.attrs.to_vec(),
            vis: self.visibility,
            field_type: self.field_type.clone(),
            name: self.name.clone(),
            default_value: self.default_value.clone(),
            id: generate_member_id(&self.name)
        }
    }

    /// Pass field to parent builder and returns it
    pub fn build(&'a mut self) -> &'a mut T {
        if self.is_static {
            self.builder.add_static_field(self.get_field());
        }
        else {
            self.builder.add_field(self.get_field());
        }
        self.builder
    }
}

impl<T: FieldCollector> AttributeCollector for FieldBuilder<'_, T> {
    fn add_attribute(&mut self, attribute: Attribute) -> &mut Self {
        self.attrs.push(attribute);
        self
    }
}