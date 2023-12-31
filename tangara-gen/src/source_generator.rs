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
                    name.push_str(&get_typeref(dep));
                    name.push('+');
                }
                name.remove(name.len() - 1);
            }
            name.push(',');
        }
        name.remove(name.len() - 1);
        name.push('>');
    }
    // TODO add lifetimes
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
            name.push_str(&get_typeref(parent));
            name.push('<');
            for generic in generics {
                name.push_str(&get_typeref(generic));
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
                name.push_str(&get_typeref(t));
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
                name.push_str(&get_typeref(arg));
                name.push(',');
            }
            if args.len() > 0 {
                // if args count > 0 then we have extra ',' that must be removed
                name.remove(name.len() - 1);
            }
            name.push(')');
            if let Some(return_type) = ret_type {
                name.push_str(" -> ");
                name.push_str(&get_typeref(return_type));
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
                result.push_str(&get_value(v));
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
                result.push_str(&get_value(v));
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

fn get_args(args: &[Argument]) -> String {
    let mut result = String::new();
    for arg in args {
        // TODO work with attributes
        match &arg.3 {
            // TODO research this section
            ArgumentKind::Default => {}
            ArgumentKind::DefaultValue(value) => {}
            ArgumentKind::Out => {}
            ArgumentKind::Ref => {
                result.push_str("mut ");
            }
        }
        result.push_str(&arg.2); // name
        result.push(':');
        result.push_str(&get_typeref(&arg.1)); // type
        result.push_str(", ");
    }
    if args.len() > 0 {
        // remove extra space ' '
        result.remove(result.len() - 1);
        // then extra ','
        result.remove(result.len() - 1);
    }
    result
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
                    TypeKind::Class(is_sealed, ctors, props, methods, parents) => {
                        // TODO add to load_ function
                        // TODO implement parents
                        self.bindings_block.push_str("struct ");
                        self.bindings_block.push_str(&get_type_name(&t));
                        self.bindings_block.push(';');
                    }
                    TypeKind::Enum(variants) => {
                        let index_before_vis = self.bindings_block.len() - if t.vis == Visibility::Public {
                            4
                        } else {
                            11
                        }; // index to insert attributes before 'pub ' or 'pub(crate) '
                        self.bindings_block.insert_str(index_before_vis, "#[derive(Ord, PartialOrd, Hash, Eq, PartialEq, Debug, Copy, Clone)]\n");
                        self.bindings_block.push_str("enum ");
                        self.bindings_block.push_str(&get_type_name(&t));
                        self.bindings_block.push_str(" {\n");
                        for v in variants {
                            self.bindings_block.push('\t');
                            self.bindings_block.push_str(&v.0);
                            self.bindings_block.push('=');
                            self.bindings_block.push_str(&get_value(&v.1));
                            self.bindings_block.push_str(",\n");
                        }
                        self.bindings_block.push('}');
                    }
                    TypeKind::EnumClass(variants, methods) => {
                        // TODO add to load_ function
                        self.bindings_block.push_str("enum ");
                        self.bindings_block.push_str(&get_type_name(&t));
                        self.bindings_block.push_str(" {\n");
                        self.bindings_block.push('}');
                    }
                    TypeKind::Interface(props, methods, parents) => {
                        // TODO implement parents
                        self.bindings_block.push_str("trait ");
                        self.bindings_block.push_str(&get_type_name(&t));
                        self.bindings_block.push_str(" {\n");
                        for method in methods {
                            self.bindings_block.push_str("\tfn ");
                            self.bindings_block.push_str(&method.name);
                            self.bindings_block.push('(');
                            // method kind cannot be static if reflection data was described properly
                            // but if it does, we can put it into Rust by writing function without self, why not..
                            if method.kind != MethodKind::Static {
                                self.bindings_block.push_str("&self, ");
                            }
                            self.bindings_block.push_str(&get_args(&method.args));
                            if method.args.len() == 0 {
                                // remove extra ', ' in the end after 'self'
                                self.bindings_block.remove(self.bindings_block.len() - 1);
                                self.bindings_block.remove(self.bindings_block.len() - 1);
                            }
                            self.bindings_block.push(')');
                            if let Some(return_type) = &method.return_type {
                                self.bindings_block.push_str(" -> ");
                                self.bindings_block.push_str(&get_typeref(&return_type));
                            }
                            self.bindings_block.push_str(";\n");
                        }
                        for prop in props {
                            let prop_name = &prop.name;
                            let type_name = &get_typeref(&prop.prop_type);

                            // generate getter
                            if self.pass_vis(&prop.getter_visibility) {
                                self.bindings_block.push_str("\tfn get_");
                                self.bindings_block.push_str(prop_name);
                                self.bindings_block.push_str("(&self) -> ");
                                self.bindings_block.push_str(type_name);
                                self.bindings_block.push_str(";\n");
                            }

                            // generate setter
                            if let Some(setter_vis) = prop.setter_visibility {
                                if self.pass_vis(&setter_vis) {
                                    self.bindings_block.push_str("\tfn set_");
                                    self.bindings_block.push_str(prop_name);
                                    self.bindings_block.push_str("(&mut self, value: ");
                                    self.bindings_block.push_str(type_name);
                                    self.bindings_block.push_str(");\n");
                                }
                            }

                        }
                        self.bindings_block.push('}');
                    }
                    TypeKind::Struct(ctors, props) => {
                        // TODO add to load_ function
                        self.bindings_block.push_str("struct ");
                        self.bindings_block.push_str(&get_type_name(&t));
                        self.bindings_block.push_str("{\n\tptr: Ptr\n}\n");
                        self.bindings_block.push_str("impl ");
                        self.bindings_block.push_str(&get_type_name(&t));
                        self.bindings_block.push_str(" {\n");
                        for ctor in ctors {
                            // TODO
                        }
                        for prop in props {
                            // TODO
                        }
                        self.bindings_block.push('}');
                    }
                    TypeKind::TypeAlias(alias) => {
                        self.bindings_block.push_str("type ");
                        self.bindings_block.push_str(&get_type_name(&t));
                        self.bindings_block.push('=');
                        self.bindings_block.push_str(&get_typeref(&alias));
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