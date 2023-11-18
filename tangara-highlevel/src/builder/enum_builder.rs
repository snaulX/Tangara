use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::builder::{generate_type_id, PackageBuilder, TypeBuilder};
use crate::{Attribute, Type, TypeRef, Value, Visibility};
use crate::TypeKind::Enum;

pub struct EnumBuilder {
    builder: Rc<RefCell<PackageBuilder>>,
    attrs: Vec<Attribute>,
    name: String,
    vis: Visibility,
    literals: HashMap<String, Value>,
}

impl EnumBuilder {
    pub fn new(builder: Rc<RefCell<PackageBuilder>>, name: &str) -> Self {
        let vis = builder.borrow().type_visibility;
        Self {
            attrs: vec![],
            builder,
            name: name.to_string(),
            vis,
            literals: HashMap::new()
        }
    }

    pub fn bitflags(self) -> BitflagsBuilder {
        BitflagsBuilder {
            builder: self
        }
    }

    pub fn literal(&mut self, literal: &str) -> &mut Self {
        self.literal_value(literal, Value::Int(self.literals.len() as i32))
    }
    pub fn literal_value(&mut self, literal: &str, value: Value) -> &mut Self {
        self.literals.insert(literal.to_string(), value);
        self
    }
}

impl TypeBuilder for EnumBuilder {
    fn add_attribute(&mut self, attr: Attribute) -> &mut Self {
        self.attrs.push(attr);
        self
    }

    fn get_type(&self) -> Type {
        Type {
            attrs: self.attrs.to_vec(),
            vis: self.vis.clone(),
            namespace: self.builder.borrow().namespace.clone(),
            name: self.name.clone(),
            id: generate_type_id(&self.name),
            kind: Enum(self.literals.clone())
        }
    }

    fn build(self) -> Type {
        let result_type = self.get_type();
        let mut builder = self.builder.borrow_mut();
        builder.types.push(result_type.clone());
        result_type
    }
}

pub struct BitflagsBuilder {
    builder: EnumBuilder,
}

impl BitflagsBuilder {
    pub fn literal(&mut self, literal: &str) -> &mut Self {
        let literals_count = self.builder.literals.len() as u32;
        let value = if literals_count == 0 {
            0
        } else {
            1 << (literals_count - 1)
        };
        self.builder.literal_value(literal, Value::UInt(value));
        self
    }
}

impl TypeBuilder for BitflagsBuilder {
    fn add_attribute(&mut self, attr: Attribute) -> &mut Self {
        self.builder.add_attribute(attr);
        self
    }

    fn get_type(&self) -> Type {
        let enum_builder = &self.builder;
        let mut attrs = self.builder.attrs.to_vec();
        attrs.push(Attribute(TypeRef::Name("Tangara.Flags".to_string()), vec![]));
        Type {
            attrs,
            vis: enum_builder.vis.clone(),
            namespace: enum_builder.builder.borrow().namespace.clone(),
            name: enum_builder.name.clone(),
            id: generate_type_id(&enum_builder.name),
            kind: Enum(enum_builder.literals.clone())
        }
    }

    fn build(self) -> Type {
        let result_type = self.get_type();
        let mut builder = self.builder.builder.borrow_mut();
        builder.types.push(result_type.clone());
        result_type
    }
}