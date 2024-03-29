use std::cell::RefCell;
use std::rc::Rc;
use crate::builder::{AttributeCollector, GenericsCollector, PackageBuilder, FieldBuilder, FieldCollector, TypeBuilder};
use crate::{Attribute, Field, generate_member_id, generate_type_id, Generics, Method, Property, Type, TypeRef, Value, Variant, Visibility};
use crate::TypeKind::{Enum, EnumClass};

pub struct EnumBuilder {
    builder: Rc<RefCell<PackageBuilder>>,
    attrs: Vec<Attribute>,
    name: String,
    namespace: String,
    vis: Visibility,
    variants: Vec<(String, Value)>,
}

impl EnumBuilder {
    pub fn new(builder: Rc<RefCell<PackageBuilder>>, name: &str) -> Self {
        let namespace = builder.borrow().get_namespace();
        let vis = builder.borrow().type_visibility;
        Self {
            attrs: vec![],
            builder,
            name: name.to_string(),
            namespace,
            vis,
            variants: vec![]
        }
    }

    pub fn bitflags(self) -> BitflagsBuilder {
        BitflagsBuilder {
            builder: self
        }
    }

    pub fn variant(&mut self, literal: &str) -> &mut Self {
        self.variant_value(literal, Value::Int(self.variants.len() as i32))
    }
    pub fn variant_value(&mut self, literal: &str, value: Value) -> &mut Self {
        self.variants.push((literal.to_string(), value));
        self
    }
}

impl TypeBuilder for EnumBuilder {
    fn add_attribute(&mut self, attr: Attribute) -> &mut Self {
        self.attrs.push(attr);
        self
    }

    fn set_visibility(&mut self, vis: Visibility) -> &mut Self {
        self.vis = vis;
        self
    }

    fn get_type(&self) -> Type {
        let namespace = self.namespace.clone();
        let name = self.name.clone();
        let mut full_name = String::with_capacity(namespace.len() + name.len() + 1);
        full_name.push_str(&namespace);
        full_name.push('.');
        full_name.push_str(&name);
        let id = generate_type_id(&full_name);
        Type {
            attrs: self.attrs.to_vec(),
            vis: self.vis.clone(),
            namespace,
            name,
            id,
            generics: Generics(vec![], vec![]),
            kind: Enum {
                variants: self.variants.clone()
            }
        }
    }

    fn build(self) -> Type {
        let result_type = self.get_type();
        let mut builder = self.builder.borrow_mut();
        builder.add_type(result_type.clone());
        result_type
    }
}

pub struct BitflagsBuilder {
    builder: EnumBuilder,
}

impl BitflagsBuilder {
    pub fn variant(&mut self, literal: &str) -> &mut Self {
        let literals_count = self.builder.variants.len() as u32;
        let value = if literals_count == 0 {
            0
        } else {
            1 << (literals_count - 1)
        };
        self.builder.variant_value(literal, Value::UInt(value));
        self
    }
}

impl TypeBuilder for BitflagsBuilder {
    fn add_attribute(&mut self, attr: Attribute) -> &mut Self {
        TypeBuilder::add_attribute(&mut self.builder, attr);
        self
    }

    fn set_visibility(&mut self, vis: Visibility) -> &mut Self {
        self.builder.set_visibility(vis);
        self
    }

    fn get_type(&self) -> Type {
        let enum_builder = &self.builder;
        let mut attrs = self.builder.attrs.to_vec();
        attrs.push(Attribute(TypeRef::Name("Tangara.Flags".to_string()), vec![]));
        let namespace = enum_builder.builder.borrow().namespace.clone();
        let name = enum_builder.name.clone();
        let mut full_name = String::with_capacity(namespace.len() + name.len() + 1);
        full_name.push_str(&namespace);
        full_name.push('.');
        full_name.push_str(&name);
        let id = generate_type_id(&full_name);
        Type {
            attrs,
            vis: enum_builder.vis.clone(),
            namespace,
            name,
            id,
            generics: Generics(vec![], vec![]),
            kind: Enum {
                variants: enum_builder.variants.clone()
            }
        }
    }

    fn build(self) -> Type {
        let result_type = self.get_type();
        let mut builder = self.builder.builder.borrow_mut();
        builder.add_type(result_type.clone());
        result_type
    }
}

pub struct EnumClassBuilder {
    builder: Rc<RefCell<PackageBuilder>>,
    attrs: Vec<Attribute>,
    name: String,
    namespace: String,
    vis: Visibility,
    variants: Vec<Variant>,
    methods: Vec<Method>,
    generics: Vec<String>,
    generics_where: Vec<(String, TypeRef)>,
}

impl EnumClassBuilder {
    pub fn new(builder: Rc<RefCell<PackageBuilder>>, name: &str) -> Self {
        let namespace = builder.borrow().get_namespace();
        let vis = builder.borrow().type_visibility;
        Self {
            attrs: vec![],
            builder,
            name: name.to_string(),
            namespace,
            vis,
            variants: vec![],
            methods: vec![],
            generics: vec![],
            generics_where: vec![]
        }
    }

    pub fn variant(&mut self, name: &str) -> VariantBuilder {
        VariantBuilder::new(self, name)
    }
}

impl TypeBuilder for EnumClassBuilder {
    fn add_attribute(&mut self, attr: Attribute) -> &mut Self {
        self.attrs.push(attr);
        self
    }

    fn set_visibility(&mut self, vis: Visibility) -> &mut Self {
        self.vis = vis;
        self
    }

    fn get_type(&self) -> Type {
        let namespace = self.namespace.clone();
        let name = self.name.clone();
        let mut full_name = String::with_capacity(namespace.len() + name.len() + 1);
        full_name.push_str(&namespace);
        full_name.push('.');
        full_name.push_str(&name);
        let id = generate_type_id(&full_name);
        Type {
            attrs: self.attrs.to_vec(),
            vis: self.vis.clone(),
            namespace,
            name,
            id,
            generics: Generics(self.generics.to_vec(), self.generics_where.to_vec()),
            kind: EnumClass {
                variants: self.variants.to_vec(),
                methods: self.methods.to_vec()
            }
        }
    }

    fn build(self) -> Type {
        let result_type = self.get_type();
        let mut builder = self.builder.borrow_mut();
        builder.add_type(result_type.clone());
        result_type
    }
}

impl GenericsCollector for EnumClassBuilder {
    fn generic(&mut self, generic: String) -> &mut Self {
        self.generics.push(generic);
        self
    }

    /// Add statement for generics `where statement.0: statement.1`.
    /// Function *panics* if first type doesn't exists in generics of this enum class.
    fn generic_where(&mut self, statement: (String, TypeRef)) -> &mut Self {
        if !self.generics.contains(&statement.0) {
            panic!(
                "Generic {} doesn't exists in this enum class, so it can't be used in 'where' statement",
                statement.0);
        }
        self.generics_where.push(statement);
        self
    }
}

/// Builder of struct variant for *'enum class'*.
/// Do not use separate from `EnumClassBuilder`.
pub struct VariantBuilder<'a> {
    builder: &'a mut EnumClassBuilder,
    attrs: Vec<Attribute>,
    vis: Visibility,
    name: String,
    fields: Vec<Field>
}

impl<'a> VariantBuilder<'a> {
    pub(crate) fn new(builder: &'a mut EnumClassBuilder, name: &str) -> Self {
        let vis = builder.builder.borrow().member_visibility;
        Self {
            builder,
            attrs: vec![],
            vis,
            name: name.to_string(),
            fields: vec![]
        }
    }

    fn set_visibility(&mut self, vis: Visibility) -> &mut Self {
        self.vis = vis;
        self
    }

    pub fn add_field(&mut self, field_type: TypeRef, name: &str) -> FieldBuilder<Self> {
        FieldBuilder::new(self, false, field_type, name)
    }

    pub fn build(self) -> &'a mut EnumClassBuilder {
        self.builder.variants.push(Variant {
            attrs: self.attrs,
            vis: self.vis,
            name: self.name.clone(),
            id: generate_member_id(&self.name),
            fields: self.fields,
        });
        self.builder
    }
}

impl<'a> FieldCollector for VariantBuilder<'a> {
    fn get_default_visibility(&self) -> Visibility {
        self.builder.builder.borrow().member_visibility
    }

    fn add_field(&mut self, field: Field) {
        self.fields.push(field);
    }

    fn add_static_field(&mut self, _: Field) {
        unreachable!("Enum class can't have static fields")
    }
}

impl<'a> AttributeCollector for VariantBuilder<'a> {
    fn add_attribute(&mut self, attribute: Attribute) -> &mut Self {
        self.attrs.push(attribute);
        self
    }
}