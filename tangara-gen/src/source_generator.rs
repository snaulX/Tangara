use std::fmt::format;
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

fn get_generics(generics: &Generics, with_where: bool) -> String {
    let mut name = String::new();
    if generics.0.len() > 0 {
        name.push('<');
        for generic in &generics.0 {
            name.push_str(generic);
            if with_where {
                let wheres = generics.1.iter().filter(|where_clause| where_clause.0 == *generic);
                if wheres.clone().count() > 0 {
                    name.push(':');
                    for (_, dep) in wheres {
                        name.push_str(&get_typeref(dep));
                        name.push('+');
                    }
                    name.remove(name.len() - 1);
                }
            }
            name.push(',');
        }
        // remove extra ',' in the end
        name.remove(name.len() - 1);
        name.push('>');
    }
    // TODO somehow handle lifetimes
    name
}

fn get_type_name(t: &Type, with_where: bool) -> String {
    let mut name = t.name.clone();
    name.push_str(&get_generics(&t.generics, with_where));
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
        // TODO work with attributes: handle lifetimes
        let type_prefix = match &arg.3 {
            ArgumentKind::Default => "",
            ArgumentKind::DefaultValue(_) => "",
            ArgumentKind::Out => "&mut ",
            ArgumentKind::Ref => "&mut ",
            ArgumentKind::In => "&"
        };
        result.push_str(&arg.2); // name
        result.push(':');
        result.push_str(type_prefix);
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

    /// Note: set `type_name` to None if you want generate property functions without a body.
    fn gen_property(&mut self, property: &Property, type_name: Option<&str>) {
        let prop_name = &property.name;
        let prop_type_name = &get_typeref(&property.prop_type);
        let mut prop_load_name = None;

        // generate getter
        if self.pass_vis(&property.getter_visibility) {
            self.bindings_block.push_str("\tfn get_");
            self.bindings_block.push_str(prop_name);
            self.bindings_block.push_str("(&self) -> ");
            self.bindings_block.push_str(prop_type_name);
            if let Some(parent_type_name) = type_name {
                // add static variable
                let getter_name = format!("{}_{}_getter", parent_type_name, prop_name);
                self.statics_block.push_str(
                    &format!("static mut {}: Option<extern \"C\" fn(Ptr) -> Ptr> = None;\n", getter_name)
                );

                // create property variable in the load body
                prop_load_name = Some(format!("{}_{}_prop", parent_type_name, prop_name));
                self.load_body.push_str("let ");
                self.load_body.push_str(&prop_load_name.clone().unwrap());
                self.load_body.push_str(" = ");
                self.load_body.push_str(parent_type_name);
                self.load_body.push_str("_type.get_property(");
                self.load_body.push_str(&property.id.to_string());
                self.load_body.push_str(");\n");
                // assign getter then
                self.load_body.push_str(
                    &format!("{} = Some({}.getter);\n", getter_name, prop_load_name.clone().unwrap())
                );

                // implement body
                self.bindings_block.push_str(" {\n\t\tunsafe {\n\t\t\tlet raw_ptr: *mut ");
                self.bindings_block.push_str(prop_type_name);
                self.bindings_block.push_str(" = (");
                self.bindings_block.push_str(&getter_name);
                self.bindings_block.push_str(".unwrap())(self as *const ");
                self.bindings_block.push_str(parent_type_name);
                self.bindings_block.push_str(" as Ptr) as *mut ");
                self.bindings_block.push_str(prop_type_name);
                self.bindings_block.push_str(";\n\t\t\tif !raw_ptr.is_null() {\n\t\t\t\t\
                *unsafe { Box::from_raw(raw_ptr) }\n\t\t\t} else {\n\t\t\t\t\
                panic!(\"Pointer of gotten property is null\")\n\t\t\t}\n\t\t}\n\t}\n");
            }
            else {
                self.bindings_block.push_str(";\n");
            }
        }

        // generate setter
        if let Some(setter_vis) = property.setter_visibility {
            if self.pass_vis(&setter_vis) {
                self.bindings_block.push_str("\tfn set_");
                self.bindings_block.push_str(prop_name);
                self.bindings_block.push_str("(&mut self, value: ");
                self.bindings_block.push_str(prop_type_name);
                self.bindings_block.push(')');
                if let Some(parent_type_name) = type_name {
                    // add static variable
                    let setter_name = format!("{}_{}_setter", parent_type_name, prop_name);
                    self.statics_block.push_str(
                        &format!("static mut {}: Option<extern \"C\" fn(Ptr, Ptr)> = None;\n", setter_name)
                    );

                    // create property variable in the load body if it doesn't exists yet
                    let prop_name_from_load = prop_load_name.unwrap_or_else(|| {
                        let prop_load_name = format!("{}_{}_prop", parent_type_name, prop_name);
                        self.load_body.push_str("let ");
                        self.load_body.push_str(&prop_load_name);
                        self.load_body.push_str(" = ");
                        self.load_body.push_str(parent_type_name);
                        self.load_body.push_str("_type.get_property(");
                        self.load_body.push_str(&property.id.to_string());
                        self.load_body.push_str(");\n");
                        prop_load_name
                    });
                    // assign setter then
                    self.load_body.push_str(
                        &format!("{} = Some({}.setter.unwrap());\n", setter_name, prop_name_from_load)
                    );

                    // implement body
                    self.bindings_block.push_str(" {\n\t\tunsafe { (");
                    self.bindings_block.push_str(&setter_name);
                    self.bindings_block.push_str(".unwrap())(self as *mut ");
                    self.bindings_block.push_str(parent_type_name);
                    self.bindings_block.push_str(" as Ptr, &value as *const ");
                    self.bindings_block.push_str(prop_type_name);
                    self.bindings_block.push_str(" as Ptr); }\n\t}\n");
                }
                else {
                    self.bindings_block.push_str(";\n");
                }
            }
        }
    }

    fn gen_drop(&mut self, t: &Type, type_load_name: &str) {
        // add static destructor variable
        let dtor_name = format!("{}_dtor", t.name);
        self.statics_block.push_str(&format!("static mut {dtor_name}: Option<FnDtor> = None;\n"));

        // assign it in the load body
        self.load_body.push_str(&format!("{dtor_name} = Some({type_load_name}.get_dtor());\n"));

        // implement Drop trait
        self.bindings_block.push_str("\nimpl");
        self.bindings_block.push_str(&get_generics(&t.generics, true));
        self.bindings_block.push_str(" Drop for ");
        self.bindings_block.push_str(&get_type_name(&t, false));
        self.bindings_block.push_str(" {\n\tfn drop(&mut self) {\n\t\tunsafe {\n\t\t\t");
        self.bindings_block.push_str(&dtor_name);
        self.bindings_block.push_str(".expect(\"Destructor wasn't loaded from library\")(self.ptr);\n\t\t}\n\t}\n}");
    }

    fn add_load_type(&mut self, t: &Type) -> String {
        let type_name = format!("{}_type", t.name);
        self.load_body.push_str(
            &format!("let {} = {}.get_type({});\n", type_name, self.package_name, t.id.to_string())
        );
        type_name
    }

    pub fn generate(&mut self) {
        self.load_body.push_str(
            &format!("let {} = ctx.get_package({});\n", self.package_name, self.package.id)
        );
        let types = self.package.types.to_vec();
        for t in types {
            if self.pass_vis(&t.vis) {
                // TODO: resolve generics later
                if t.generics.0.len() > 0 {
                    continue; // we can't resolve generics for now
                }

                if t.vis == Visibility::Public {
                    self.bindings_block.push_str("pub ");
                }
                else {
                    self.bindings_block.push_str("pub(crate) ");
                }
                match &t.kind {
                    TypeKind::Class(is_sealed, ctors, props, methods, parents) => {
                        let class_load_name = self.add_load_type(&t);
                        // TODO implement parents
                        // TODO do something with 'is_sealed'
                        self.bindings_block.push_str(&format!(r#"struct {} {{
    ptr: Ptr
}}

impl{} {} {{
"#, get_type_name(&t, true), get_generics(&t.generics, true), get_type_name(&t, false)));
                        // first was type name with generics and where Type<T: Kek>
                        // second was generics with where <T: Kek>
                        // third was type name with generics without where Type<T>
                        // TODO implement ctors
                        for prop in props {
                            self.gen_property(&prop, Some(&t.name));
                        }
                        // TODO implement methods
                        self.bindings_block.push('}');
                        self.gen_drop(&t, &class_load_name);
                    }
                    TypeKind::Enum(variants) => {
                        let index_before_vis = self.bindings_block.len() - if t.vis == Visibility::Public {
                            4
                        } else {
                            11
                        }; // index to insert attributes before 'pub ' or 'pub(crate) '
                        self.bindings_block.insert_str(index_before_vis, "#[derive(Ord, PartialOrd, Hash, Eq, PartialEq, Debug, Copy, Clone)]\n");
                        self.bindings_block.push_str("enum ");
                        self.bindings_block.push_str(&get_type_name(&t, true));
                        self.bindings_block.push_str(" {\n");
                        for v in variants {
                            self.bindings_block.push_str(&format!("\t{} = {},\n", &v.0, &get_value(&v.1)));
                        }
                        self.bindings_block.push('}');
                    }
                    TypeKind::EnumClass(variants, methods) => {
                        let enum_load_name = self.add_load_type(&t);
                        self.bindings_block.push_str("enum ");
                        self.bindings_block.push_str(&get_type_name(&t, true));
                        self.bindings_block.push_str(" {\n");
                        // TODO implement variants
                        // TODO implement methods
                        self.bindings_block.push('}');
                    }
                    TypeKind::Interface(props, methods, parents) => {
                        // TODO implement parents
                        self.bindings_block.push_str("trait ");
                        self.bindings_block.push_str(&get_type_name(&t, true));
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
                            self.gen_property(&prop, None);
                        }
                        self.bindings_block.push('}');
                    }
                    TypeKind::Struct(ctors, props) => {
                        let struct_load_name = self.add_load_type(&t);
                        self.bindings_block.push_str(&format!(r#"struct {} {{
    ptr: Ptr
}}

impl{} {} {{
"#, get_type_name(&t, true), get_generics(&t.generics, true), get_type_name(&t, false)));
                        // first was type name with generics and where Type<T: Kek>
                        // second was generics with where <T: Kek>
                        // third was type name with generics without where Type<T>
                        let mut ctor_counter = 0;
                        let mut default_ctor_index = None;
                        for ctor in ctors {
                            if ctor.args.len() == 0 {
                                default_ctor_index = Some(ctor_counter);
                            }
                            let ctor_name = if let Some(ctor_fn_name) = RUST_STD_LIB.get_fn_name(&ctor.attrs) {
                                // get name from ConstructorFnName attribute if it exists
                                ctor_fn_name
                            } else {
                                // or create something like 'new0'
                                format!("{}{}", self.config.ctor_name, ctor_counter)
                            };
                            self.bindings_block.push_str("\tfn ");
                            self.bindings_block.push_str(&ctor_name);
                            self.bindings_block.push('(');
                            self.bindings_block.push_str(&get_args(&ctor.args));
                            self.bindings_block.push_str(") -> Self {\n");
                            // TODO body
                            self.bindings_block.push('}');
                            ctor_counter += 1;
                        }
                        for prop in props {
                            self.gen_property(&prop, Some(&t.name));
                        }
                        self.bindings_block.push('}');
                        self.gen_drop(&t, &struct_load_name);

                        // implement Default trait for empty constructor
                        if self.config.generate_default {
                            if let Some(ctor_index) = default_ctor_index {
                                self.bindings_block.push_str("\nimpl");
                                self.bindings_block.push_str(&get_generics(&t.generics, true));
                                self.bindings_block.push_str(" Default for ");
                                self.bindings_block.push_str(&get_type_name(&t, false));
                                self.bindings_block.push_str(" {\n\tfn default() -> Self {\n");
                                // TODO implement default() body
                                self.bindings_block.push_str("\t}\n}");
                            }
                        }
                    }
                    TypeKind::TypeAlias(alias) => {
                        self.bindings_block.push_str("type ");
                        self.bindings_block.push_str(&get_type_name(&t, true));
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
use tangara::context::{FnDtor, Context, Ptr, Fn};

"#.to_string();
        self.statics_block.push('\n');
        let mut load_body = self.load_body.replace("\n", "\n\t\t");
        load_body.remove(load_body.len() - 1); // remove last extra '\t'
        let pkg_name = self.package.name;
        let load_fn = format!("\npub fn load_{pkg_name}(ctx: &Context) {{\n\tunsafe {{\n\t\t{load_body}}}\n}}\n");
        std::fs::write(path, String::from_iter([disclaimer, self.statics_block, self.bindings_block, load_fn]))
    }
}