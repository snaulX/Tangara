use std::collections::HashMap;
use crate::builder::{generate_type_id, PackageBuilder, TypeBuilder};
use crate::{Attribute, Type, TypeRef, Value, Visibility};
use crate::TypeKind::Enum;

pub struct EnumBuilder<'a> {
    builder: &'a mut PackageBuilder,
    name: String,
    vis: Visibility,
    literals: HashMap<String, Value>,
}

impl<'a> EnumBuilder<'a> {
    pub(crate) fn new(builder: &'a mut PackageBuilder, name: &str) -> Self {
        let vis = builder.type_visibility;
        Self {
            builder,
            name: name.to_string(),
            vis,
            literals: HashMap::new()
        }
    }

    pub fn bitflags(mut self) -> BitflagsBuilder<'a> {
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

impl<'a> TypeBuilder for EnumBuilder<'a> {
    fn get_type(&self) -> Type {
        Type {
            vis: self.vis.clone(),
            namespace: self.builder.namespace.clone(),
            name: self.name.clone(),
            id: generate_type_id(&self.name),
            attrs: vec![],
            kind: Enum(self.literals.clone())
        }
    }

    fn build(&mut self) -> &mut PackageBuilder {
        self.builder.types.push(self.get_type());
        self.builder
    }
}

pub struct BitflagsBuilder<'a> {
    builder: EnumBuilder<'a>,
}

impl<'a> BitflagsBuilder<'a> {
    pub fn literal(&mut self, literal: &str) -> &mut Self {
        let literals_count = (self.builder.literals.len() as u32);
        let value = if literals_count == 0 {
            0
        } else {
            1 << (literals_count - 1)
        };
        self.builder.literal_value(literal, Value::UInt(value));
        self
    }
}

impl<'a> TypeBuilder for BitflagsBuilder<'a> {
    fn get_type(&self) -> Type {
        let enum_builder = &self.builder;
        Type {
            vis: enum_builder.vis.clone(),
            namespace: enum_builder.builder.namespace.clone(),
            name: enum_builder.name.clone(),
            id: generate_type_id(&enum_builder.name),
            attrs: vec![
                Attribute(TypeRef::Name("Flags".to_string()), vec![])
            ],
            kind: Enum(enum_builder.literals.clone())
        }
    }

    fn build(&mut self) -> &mut PackageBuilder {
        self.builder.builder.types.push(self.get_type());
        self.builder.builder
    }
}