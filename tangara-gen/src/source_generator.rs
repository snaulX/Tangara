use std::path::Path;
use tangara_highlevel::*;
use crate::rust_generator::Config;
use crate::RUST_STD_LIB;

pub struct SourceGenerator {
    config: Config,
    package: Package,
    statics_block: String,
    bindings_block: String,
    load_body: String,
    package_name: String
}

impl SourceGenerator {
    pub(crate) fn new(package: Package, config: Config) -> Self {
        let package_name = format!("{}_package", package.name);
        Self {
            config,
            package,
            statics_block: String::new(),
            bindings_block: String::new(),
            load_body: String::new(),
            package_name
        }
    }

    fn pass_vis(&self, vis: &Visibility) -> bool {
        let vis = vis.clone();
        vis == Visibility::Public || (self.config.enable_internal && vis == Visibility::Internal)
    }

    fn get_type_name(t: &Type) -> String {
        let mut name = t.name.clone();
        if t.generics.0.len() > 0 {
            name.push('<');
            for generic in &t.generics.0 {
                name.push_str(generic);
                let wheres = t.generics.1.iter().filter(|where_clause| where_clause.0 == *generic);
                if wheres.clone().count() > 0 {
                    name.push(':');
                    for (_, dep) in wheres {
                        name.push_str(&Self::get_typeref(dep));
                        name.push('+');
                    }
                    name.remove(name.len() - 1);
                }
                name.push(',');
            }
            name.remove(name.len() - 1);
            name.push('>');
        }
        name
    }

    fn get_typeref(typeref: &TypeRef) -> String {
        let mut name = String::new();
        match typeref {
            TypeRef::Name(type_name) => {
                // replace '.' from namespaces for '::' for Rust modules
                name.push_str(&type_name.replace(".", "::"));
            }
            TypeRef::Id(id) => {
                // TODO resolve type
            }
            TypeRef::Generic(parent, generics) => {
                name.push_str(&Self::get_typeref(parent));
                name.push('<');
                for generic in generics {
                    name.push_str(&Self::get_typeref(generic));
                    name.push(',');
                }
                if generics.len() > 0 {
                    // if generics count > 0 then we have extra ',' that must be removed
                    name.remove(name.len() - 1);
                }
                name.push('>');
            }
            TypeRef::Tuple(types) => {
                name.push('(');
                for t in types {
                    name.push_str(&Self::get_typeref(t));
                    name.push(',');
                }
                if types.len() > 0 {
                    // if types count > 0 then we have extra ',' that must be removed
                    name.remove(name.len() - 1);
                }
                name.push(')');
            }
            TypeRef::Fn(ret_type, args) => {
                name.push_str("fn(");
                for arg in args {
                    name.push_str(&Self::get_typeref(arg));
                    name.push(',');
                }
                if args.len() > 0 {
                    // if args count > 0 then we have extra ',' that must be removed
                    name.remove(name.len() - 1);
                }
                name.push(')');
                if let Some(return_type) = ret_type {
                    name.push_str(" -> ");
                    name.push_str(&Self::get_typeref(return_type));
                }
            }
        }
        name
    }

    fn get_value(value: &Value) -> String {
        match value {
            Value::Null => "None".to_string(),
            Value::Bool(bool_value) => bool_value.to_string(),
            Value::Byte(byte_value) => byte_value.to_string(),
            Value::Short(short_value) => short_value.to_string(),
            Value::Int(int_value) => int_value.to_string(),
            Value::Long(long_value) => long_value.to_string(),
            Value::SByte(byte_value) => byte_value.to_string(),
            Value::UShort(short_value) => short_value.to_string(),
            Value::UInt(int_value) => int_value.to_string(),
            Value::ULong(long_value) => long_value.to_string(),
            Value::Float(long_value) => long_value.to_string(),
            Value::Double(long_value) => long_value.to_string(),
            Value::String(string_value) => string_value.to_string(),
            Value::Array(array_value) => {
                let mut result = String::new();
                result.push('[');
                for v in array_value {
                    result.push_str(&Self::get_value(v));
                    result.push(',');
                }
                if array_value.len() > 0 {
                    // if array values count > 0 then we have extra ',' that must be removed
                    result.remove(result.len() - 1);
                }
                result.push(']');
                result
            }
            Value::Tuple(tuple_value) => {
                let mut result = String::new();
                result.push('(');
                for v in tuple_value {
                    result.push_str(&Self::get_value(v));
                    result.push(',');
                }
                if tuple_value.len() > 0 {
                    // if array values count > 0 then we have extra ',' that must be removed
                    result.remove(result.len() - 1);
                }
                result.push(')');
                result
            }
            Value::Object(object_value) => unimplemented!("Object values can't be converted to string for now")
        }
    }

    pub fn generate(&mut self) {
        self.load_body.push_str(
            &format!("let {} = ctx.get_package({});\n", self.package_name, self.package.id)
        );
        let types = self.package.types.to_vec();
        for t in types {
            if self.pass_vis(&t.vis) {
                if t.vis == Visibility::Public {
                    self.bindings_block.push_str("pub ");
                }
                else {
                    self.bindings_block.push_str("pub(crate) ");
                }
                match &t.kind {
                    TypeKind::Class(_, _, _, _, _) => {
                        self.bindings_block.push_str("struct ");
                        self.bindings_block.push_str(&Self::get_type_name(&t));
                        self.bindings_block.push(';');
                    }
                    TypeKind::Enum(literals) => {
                        self.bindings_block.push_str("enum ");
                        self.bindings_block.push_str(&Self::get_type_name(&t));
                        self.bindings_block.push_str(" {\n");
                        for l in literals {
                            self.bindings_block.push('\t');
                            self.bindings_block.push_str(&l.0);
                            self.bindings_block.push('=');
                            self.bindings_block.push_str(&Self::get_value(&l.1));
                            self.bindings_block.push_str(",\n");
                        }
                        self.bindings_block.push('}');
                    }
                    TypeKind::EnumClass(_, _) => {
                        self.bindings_block.push_str("enum ");
                        self.bindings_block.push_str(&Self::get_type_name(&t));
                        self.bindings_block.push_str(" {\n");
                        self.bindings_block.push('}');
                    }
                    TypeKind::Interface(_, _, _) => {
                        self.bindings_block.push_str("trait ");
                        self.bindings_block.push_str(&Self::get_type_name(&t));
                        self.bindings_block.push_str(" {\n");
                        self.bindings_block.push('}');
                    }
                    TypeKind::Struct(_, _) => {
                        self.bindings_block.push_str("struct ");
                        self.bindings_block.push_str(&Self::get_type_name(&t));
                        self.bindings_block.push(';');
                    }
                    TypeKind::TypeAlias(alias) => {
                        self.bindings_block.push_str("type ");
                        self.bindings_block.push_str(&Self::get_type_name(&t));
                        self.bindings_block.push('=');
                        self.bindings_block.push_str(&Self::get_typeref(&alias));
                        self.bindings_block.push(';');
                    }
                }
                self.bindings_block.push('\n');
            }
        }
    }

    pub fn write_to<P: AsRef<Path>>(mut self, path: P) -> std::io::Result<()> {
        self.generate();
        let disclaimer = r#"// This file was generated by tangara-gen
// All changes in this file will discard after rebuilding project
use tangara::context::{Context, Ptr, Property};
"#.to_string();
        let mut load_body = self.load_body.replace("\n", "\n\t");
        load_body.remove(load_body.len() - 1); // remove last extra '\t'
        let pkg_name = self.package.name;
        let load_fn = format!("pub fn load_{pkg_name}(ctx: &Context) {{\n\t{load_body}}}\n");
        std::fs::write(path, String::from_iter([disclaimer, self.statics_block, self.bindings_block, load_fn]))
    }
}