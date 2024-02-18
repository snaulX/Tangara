use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use crate::builder::{FieldBuilder, FieldCollector, GenericsCollector, PackageBuilder, TypeBuilder};
use crate::*;
use crate::builder::constructor_builder::*;
use crate::builder::method_builder::*;
use crate::builder::property_builder::*;
use crate::TypeKind::Class;

pub struct ClassBuilder {
    builder: Rc<RefCell<PackageBuilder>>,
    attrs: Vec<Attribute>,
    name: String,
    namespace: String,
    vis: Visibility,
    constructors: Vec<Constructor>,
    properties: Vec<Property>,
    fields: Vec<Field>,
    static_properties: Vec<Property>,
    static_fields: Vec<Field>,
    methods: Vec<Method>,
    parents: Vec<TypeRef>,
    generics: Vec<String>,
    generics_where: Vec<(String, TypeRef)>,
    sealed: bool
}

impl ClassBuilder {
    pub fn new(builder: Rc<RefCell<PackageBuilder>>, name: &str) -> Self {
        let namespace = builder.borrow().get_namespace();
        let vis = builder.borrow().type_visibility;
        Self {
            builder,
            attrs: vec![],
            name: name.to_string(),
            namespace,
            vis,
            constructors: vec![],
            properties: vec![],
            fields: vec![],
            static_properties: vec![],
            static_fields: vec![],
            methods: vec![],
            parents: vec![],
            generics: vec![],
            generics_where: vec![],
            sealed: true
        }
    }

    pub fn inherits(&mut self, parent: TypeRef) -> &mut Self {
        self.parents.push(parent);
        self
    }

    /// Make class open to inherit from
    pub fn open(&mut self) -> &mut Self {
        self.sealed = false;
        self
    }

    pub fn add_constructor(&mut self) -> ConstructorBuilder<Self> {
        ConstructorBuilder::new(self)
    }

    pub fn add_property(&mut self, prop_type: TypeRef, name: &str) -> PropertyBuilder<Self> {
        PropertyBuilder::new(self, false, prop_type, name)
    }
    
    pub fn add_field(&mut self, field_type: TypeRef, name: &str) -> FieldBuilder<Self> {
        FieldBuilder::new(self, false, field_type, name)
    }

    pub fn add_static_property(&mut self, prop_type: TypeRef, name: &str) -> PropertyBuilder<Self> {
        PropertyBuilder::new(self, true, prop_type, name)
    }

    pub fn add_static_field(&mut self, field_type: TypeRef, name: &str) -> FieldBuilder<Self> {
        FieldBuilder::new(self, true, field_type, name)
    }

    pub fn add_method(&mut self, name: &str) -> MethodBuilder<Self> {
        MethodBuilder::new(self, name)
    }
}

impl TypeBuilder for ClassBuilder {
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
            kind: Class {
                is_sealed: self.sealed,
                constructors: self.constructors.to_vec(),
                properties: self.properties.to_vec(),
                fields: self.fields.to_vec(),
                static_properties: self.static_properties.to_vec(),
                static_fields: self.static_fields.to_vec(),
                methods: self.methods.to_vec(),
                parents: self.parents.to_vec(),
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

impl GenericsCollector for ClassBuilder {
    fn generic(&mut self, generic: String) -> &mut Self {
        self.generics.push(generic);
        self
    }

    /// Add statement for generics `where statement.0: statement.1`.
    /// Function *panics* if first type doesn't exists in generics of this class.
    fn generic_where(&mut self, statement: (String, TypeRef)) -> &mut Self {
        if !self.generics.contains(&statement.0) {
            panic!(
                "Generic {} doesn't exists in this class, so it can't be used in 'where' statement",
                statement.0);
        }
        self.generics_where.push(statement);
        self
    }
}

impl ConstructorCollector for ClassBuilder {
    fn get_default_visibility(&self) -> Visibility {
        self.builder.borrow().constructor_visibility
    }

    fn add_constructor(&mut self, constructor: Constructor) {
        self.constructors.push(constructor)
    }
}

impl PropertyCollector for ClassBuilder {
    fn get_default_visibility(&self) -> Visibility {
        self.builder.borrow().member_visibility
    }

    fn add_property(&mut self, property: Property) {
        self.properties.push(property)
    }

    fn add_static_property(&mut self, property: Property) {
        self.static_properties.push(property)
    }
}

impl FieldCollector for ClassBuilder {
    fn get_default_visibility(&self) -> Visibility {
        self.builder.borrow().member_visibility
    }

    fn add_field(&mut self, field: Field) {
        self.fields.push(field)
    }

    fn add_static_field(&mut self, field: Field) {
        self.static_fields.push(field)
    }
}

impl MethodCollector for ClassBuilder {
    fn get_default_visibility(&self) -> Visibility {
        self.builder.borrow().method_visibility
    }

    fn get_supported_kinds(&self) -> HashSet<MethodKind> {
        if self.sealed {
            HashSet::from([MethodKind::Default, MethodKind::Static])
        } else {
            HashSet::from([MethodKind::Default, MethodKind::Static, MethodKind::Virtual])
        }
    }

    fn add_method(&mut self, method: Method) {
        self.methods.push(method)
    }
}