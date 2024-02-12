use std::path::Path;
use std::string::ToString;
use once_cell::sync::Lazy;
use tangara_highlevel::*;
use crate::rust_generator::Config;
use crate::RUST_STD_LIB;

static RUST_NAMING: Lazy<NamingConventions> = Lazy::new(|| NamingConventions::rust());
// We need this list for excluding these types from naming checks (it's not using Pascal Case, so it causes errors)
static PRIMITIVE_TYPES: [&str; 12] = [
    "bool",
    "str",
    "char",
    "i8",
    "u8",
    "i16",
    "u16",
    "i32",
    "u32",
    "i64",
    "u64",
    "usize",
];

pub struct SourceGenerator {
    config: Config,
    package: Package,
    naming: NamingConventions,
    statics_block: String,
    bindings_block: String,
    load_body: String,
    package_name: String
}

fn get_generics(generics: &Generics, attrs: &[Attribute], naming: &NamingConventions, with_where: bool) -> String {
    let mut name = String::new();
    if generics.0.len() > 0 {
        name.push('<');
        for lifetime in RUST_STD_LIB.get_lifetimes(&attrs) {
            name.push_str(&format!("'{}, ", lifetime));
        }
        for generic in &generics.0 {
            name.push_str(generic);
            if with_where {
                let lifetimes = RUST_STD_LIB.get_generic_lifetimes(&attrs, generic);
                if lifetimes.len() > 0{
                    name.push(':');
                    for lt in &lifetimes {
                        name.push_str(&format!("'{} + ", lt));
                    }
                    // remove extra ' + ' in the end
                    name.remove(name.len() - 1);
                    name.remove(name.len() - 1);
                    name.remove(name.len() - 1);
                }
                let wheres = generics.1.iter().filter(|where_clause| where_clause.0 == *generic);
                if wheres.clone().count() > 0 {
                    if lifetimes.len() > 0 {
                        // we have lifetimes before wheres
                        name.push('+');
                    }
                    else {
                        // we don't have lifetimes before wheres, so we don't have ':' now
                        name.push(':');
                    }
                    for (_, dep) in wheres {
                        name.push_str(&get_typeref(dep, naming));
                        name.push('+');
                    }
                    // remove extra '+' in the end
                    name.remove(name.len() - 1);
                }
            }
            name.push_str(", ");
        }
        // remove extra ', ' in the end
        name.remove(name.len() - 1);
        name.remove(name.len() - 1);
        name.push('>');
    }
    // TODO somehow handle lifetimes
    name
}

fn get_type_name(t: &Type, naming: &NamingConventions, with_where: bool) -> String {
    let mut name = if let &TypeKind::Interface { .. } = &t.kind {
        RUST_NAMING.interface.from(&t.name, &naming.interface).unwrap()
    } else {
        RUST_NAMING.base_type.from(&t.name, &naming.base_type).unwrap()
    };
    name.push_str(&get_generics(&t.generics, &t.attrs, naming, with_where));
    name
}

fn get_typeref(typeref: &TypeRef, naming: &NamingConventions) -> String {
    let mut name = String::new();
    match typeref {
        TypeRef::Name(type_name) => {
            // if it's a primitive type (so it doesn't follow a pascal case), we don't check naming
            if PRIMITIVE_TYPES.contains(&type_name.as_str()) {
                name.push_str(type_name);
            }
            else {
                let converted = RUST_NAMING.convert_type(type_name, naming).unwrap();
                name.push_str(&converted);
            }
        }
        TypeRef::Id(id) => {
            // TODO resolve type
        }
        TypeRef::Generic(parent, generics) => {
            name.push_str(&get_typeref(parent, naming));
            name.push('<');
            for generic in generics {
                name.push_str(&get_typeref(generic, naming));
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
                name.push_str(&get_typeref(t, naming));
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
                name.push_str(&get_typeref(arg, naming));
                name.push(',');
            }
            if args.len() > 0 {
                // if args count > 0 then we have extra ',' that must be removed
                name.remove(name.len() - 1);
            }
            name.push(')');
            if let Some(return_type) = ret_type {
                name.push_str(" -> ");
                name.push_str(&get_typeref(return_type, naming));
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

fn get_args(args: &[Argument], naming: &NamingConventions) -> String {
    let mut result = String::new();
    for arg in args {
        let lifetime = if let Some(lt) = RUST_STD_LIB.get_lifetime(&arg.0) {
            format!("'{} ", lt)
        } else {
            String::new()
        };
        let type_prefix = match &arg.3 {
            ArgumentKind::Default => "".to_string(),
            ArgumentKind::DefaultValue(_) => "".to_string(),
            ArgumentKind::Out => format!("&{}mut ", lifetime),
            ArgumentKind::Ref => format!("&{}mut ", lifetime),
            ArgumentKind::In => format!("&{}", lifetime)
        };
        result.push_str(&RUST_NAMING.parameter.from(&arg.2, &naming.parameter).unwrap()); // name
        result.push(':');
        result.push_str(&type_prefix);
        result.push_str(&get_typeref(&arg.1, naming)); // type
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
        let mut package_naming = NamingConventions::rust();
        package_naming.package_divider = "_".to_string();
        let package_name = format!("{}_package", package_naming.convert_package(&package.name, &package.naming).unwrap());
        let naming = package.naming.clone();
        Self {
            config,
            package,
            naming,
            statics_block: String::new(),
            bindings_block: String::new(),
            load_body: String::new(),
            package_name
        }
    }

    /// Checks visibility for pass generating
    fn pass_vis(&self, vis: &Visibility) -> bool {
        let vis = vis.clone();
        vis == Visibility::Public || (self.config.enable_internal && vis == Visibility::Internal)
    }

    fn gen_vis(&mut self, vis: &Visibility) {
        if *vis == Visibility::Public {
            self.bindings_block.push_str("pub ");
        }
        else {
            self.bindings_block.push_str("pub(crate) ");
        }
    }

    /// Returns string of arguments you should pass to function in bindings
    fn gen_args(&mut self, args: &[Argument], with_self: bool) -> String {
        if args.len() > 0 || with_self {
            let mut args_assign = Vec::with_capacity(args.len() + 1);
            let mut args_size = String::new();
            if with_self {
                args_size.push_str("std::mem::size_of::<Ptr>()");
                args_assign.push("*(args_ptr as *mut Ptr) = self.ptr;".to_string());
            }
            for arg in args {
                let type_prefix = match &arg.3 {
                    ArgumentKind::Default => "",
                    ArgumentKind::DefaultValue(_) => "",
                    ArgumentKind::Out => "&mut ",
                    ArgumentKind::Ref => "&mut ",
                    ArgumentKind::In => "&"
                }.to_string();
                let arg_type = [type_prefix, get_typeref(&arg.1, &self.naming)].concat();
                if args_size.len() == 0 {
                    args_assign.push(format!("*(args_ptr as *mut {}) = {};", arg_type, arg.2));
                } else {
                    args_assign.push(format!("*(args_ptr.add({}) as *mut {}) = {};", args_size, arg_type, arg.2));
                    args_size.push_str(" + ");
                }
                args_size.push_str(&format!("std::mem::size_of::<{}>()", arg_type));
            }
            self.bindings_block.push_str(
                &format!(r#"
                let args_size = {args_size};
                let mut args_buf = vec![0u8; args_size];
                let args_ptr = args_buf.as_mut_ptr();
                unsafe {{
                    {}
                }}"#, args_assign.join("\n\t\t\t\t\t"))
            );
            "args_size, args_ptr".to_string()
        }
        else {
            "0, std::ptr::null_mut()".to_string()
        }
    }

    /// Note: set `type_name` to None if you want to generate property functions without a body.
    fn gen_property(&mut self, property: &Property, type_name: Option<&str>) {
        let prop_name = &if property.getter_visibility == Visibility::Public {
            RUST_NAMING.property.from(&property.name, &self.naming.property)
        } else {
            RUST_NAMING.private_field.from(&property.name, &self.naming.private_field)
        }.unwrap();
        let prop_type_name = &get_typeref(&property.prop_type, &self.naming);
        let mut prop_load_name = None;

        // generate getter
        if self.pass_vis(&property.getter_visibility) {
            self.bindings_block.push('\t');
            self.gen_vis(&property.getter_visibility);
            self.bindings_block.push_str("fn get_");
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
                self.bindings_block.push_str(" = ");
                self.bindings_block.push_str(&getter_name);
                self.bindings_block.push_str(".unwrap()(self.ptr) as *mut ");
                self.bindings_block.push_str(prop_type_name);
                self.bindings_block.push_str(";\n\t\t\tif !raw_ptr.is_null() {\n\t\t\t\t\
                *Box::from_raw(raw_ptr)\n\t\t\t} else {\n\t\t\t\t\
                panic!(\"Pointer of gotten property is null\")\n\t\t\t}\n\t\t}\n\t}\n");
            }
            else {
                self.bindings_block.push_str(";\n");
            }
        }

        // generate setter
        if let Some(setter_vis) = property.setter_visibility {
            if self.pass_vis(&setter_vis) {
                self.bindings_block.push('\t');
                self.gen_vis(&setter_vis);
                self.bindings_block.push_str("fn set_");
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
                    self.bindings_block.push_str(" {\n\t\tunsafe { ");
                    self.bindings_block.push_str(&setter_name);
                    self.bindings_block.push_str(".unwrap()(self.ptr, &value as *const ");
                    self.bindings_block.push_str(prop_type_name);
                    self.bindings_block.push_str(" as Ptr); }\n\t}\n");
                }
                else {
                    self.bindings_block.push_str(";\n");
                }
            }
        }
    }

    /// Returns constructor's function name
    fn gen_ctor(&mut self, ctor: &Constructor, index: u32, type_name: &str) -> String {
        if self.pass_vis(&ctor.vis) {
            let ctor_load_name = format!("{}_ctor{}", type_name, index);
            self.statics_block.push_str(
                &format!("static mut {}: Option<Fn> = None;\n", ctor_load_name)
            );
            self.load_body.push_str(
                &format!("{} = Some({}_type.get_ctor({}).clone());\n", ctor_load_name, type_name, index)
            );

            let ctor_name = if let Some(ctor_fn_name) = RUST_STD_LIB.get_fn_name(&ctor.attrs) {
                // get name from ConstructorFnName attribute if it exists
                ctor_fn_name
            } else {
                // or create something like 'new0'
                [self.config.ctor_name.clone(), index.to_string()].concat()
            };
            self.bindings_block.push('\t');
            self.gen_vis(&ctor.vis);
            self.bindings_block.push_str("fn ");
            self.bindings_block.push_str(&ctor_name);
            self.bindings_block.push('(');
            self.bindings_block.push_str(&get_args(&ctor.args, &self.naming));
            self.bindings_block.push_str(
                &format!(") -> Self {{\n\t\tunsafe {{\n\t\t\tif let Some(ctor_func) = {} {{", ctor_load_name)
            );
            // we don't join these two bindings' push_str calls into one because self.gen_args()
            // called below in format generating code to bindings block between these two
            let args = self.gen_args(&ctor.args, false);
            self.bindings_block.push_str(
                &format!(r#"
                let this = ctor_func({});
                if !this.is_null() {{
                    Self {{
                        ptr: this
                    }}
                }} else {{
                    panic!("Pointer of constructor result is null")
                }}
            }}
            else {{
                panic!("Constructor wasn't loaded")
            }}
        }}
    }}
"#, args)
            );
            ctor_name
        }
        else {
            "<ERRROR CONSTRUCTOR NAME>".to_string()
        }
    }

    fn gen_method(&mut self, method: &Method, type_name: &str) {
        if self.pass_vis(&method.vis) {
            let method_name = &RUST_NAMING.method.from(&method.name, &self.naming.method).unwrap();

            self.bindings_block.push('\t');
            // aware that if method is abstract - it's for traits, so it can't have visibility modifier
            if method.kind != MethodKind::Abstract {
                self.gen_vis(&method.vis);
            }
            let (return_type, return_type_block) = if let Some(ret_type) = &method.return_type {
                let prefix = RUST_STD_LIB.get_return_prefix(&method.attrs).unwrap_or_default();
                let core = [prefix, get_typeref(ret_type, &self.naming)].concat();
                (core.clone(), [" -> ", &core].concat())
            } else {
                (String::new(), String::new())
            };
            // omg, maybe I should rewrite these if'es
            let self_block = if method.kind == MethodKind::Static {
                ""
            } else if RUST_STD_LIB.is_reference(&method.attrs) {
                if RUST_STD_LIB.is_mutable(&method.attrs) {
                    "&mut self"
                }
                else {
                    "&self"
                }
            } else if RUST_STD_LIB.is_mutable(&method.attrs) {
                "mut self"
            } else {
                "self"
            };
            let args_block = if self_block.len() > 0 && method.args.len() > 0 {
                [self_block, ", ", &get_args(&method.args, &self.naming)].concat()
            } else {
                [self_block, &get_args(&method.args, &self.naming)].concat()
            };
            self.bindings_block.push_str(
                &format!("fn {method_name}({args_block}){return_type_block}")
            );
            // TODO don't forget about generics

            if method.kind != MethodKind::Abstract {
                // generate implementation
                let method_load_name = format!("{}_{}", type_name, method_name);
                self.statics_block.push_str(
                    &format!("static mut {}: Option<Fn> = None;\n", method_load_name)
                );
                self.load_body.push_str(
                    &format!("{} = Some({}_type.get_method({}).clone());\n", method_load_name, type_name, method.id)
                );

                self.bindings_block.push_str(
                    &format!(" {{\n\t\tunsafe {{\n\t\t\tif let Some(method_func) = {} {{", method_load_name)
                );
                // we don't join these two bindings' push_str calls into one because self.gen_args()
                // called below in format generating code to bindings block between these two
                let args = self.gen_args(&method.args, self_block.len() > 0);
                if method.return_type.is_none() {
                    self.bindings_block.push_str(&format!("\n\t\t\t\tmethod_func({});", args));
                } else {
                    self.bindings_block.push_str(
                        &format!(r#"
                let raw_ptr = method_func({});
                if !raw_ptr.is_null() {{
                    *Box::from_raw(raw_ptr as *mut {})
                }} else {{
                    panic!("Pointer of method result is null")
                }}"#, args, return_type)
                    );
                }
                self.bindings_block.push_str(r#"
            }
            else {
                panic!("Constructor wasn't loaded")
            }
        }
    }
"#);
            } else {
                self.bindings_block.push(';');
                self.bindings_block.push('\n');
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
        self.bindings_block.push_str("\n\nimpl");
        self.bindings_block.push_str(&get_generics(&t.generics, &t.attrs, &self.naming, true));
        self.bindings_block.push_str(" Drop for ");
        self.bindings_block.push_str(&get_type_name(&t, &self.naming, false));
        self.bindings_block.push_str(" {\n\tfn drop(&mut self) {\n\t\tunsafe {\n\t\t\t");
        self.bindings_block.push_str(&dtor_name);
        self.bindings_block.push_str(".expect(\"Destructor wasn't loaded from library\")(self.ptr);\n\t\t}\n\t}\n}");
    }

    fn gen_default(&mut self, t: &Type, ctor_name: &str) {
        if self.config.generate_default {
            self.bindings_block.push_str("\n\nimpl");
            self.bindings_block.push_str(&get_generics(&t.generics, &t.attrs, &self.naming, true));
            self.bindings_block.push_str(" Default for ");
            self.bindings_block.push_str(&get_type_name(&t, &self.naming, false));
            self.bindings_block.push_str(" {\n\tfn default() -> Self {\n\t\tunsafe {\n\t\t\t");
            self.bindings_block.push_str(&t.name);
            self.bindings_block.push_str("::");
            self.bindings_block.push_str(ctor_name);
            self.bindings_block.push_str("()\n\t\t}\n\t}\n}");
        }
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
                self.gen_vis(&t.vis);

                match &t.kind {
                    TypeKind::Class { is_sealed, constructors, properties, methods, parents} => {
                        let class_load_name = self.add_load_type(&t);
                        // TODO implement fields
                        // TODO implement parents
                        // TODO do something with 'is_sealed'
                        self.bindings_block.push_str(&format!(r#"struct {} {{
    ptr: Ptr
}}

impl{} {} {{
"#,
                                                              get_type_name(&t, &self.naming, true),
                                                              get_generics(&t.generics, &t.attrs, &self.naming, true),
                                                              get_type_name(&t, &self.naming, false)
                        )); // end of push_str(format!(/*..*/));
                        // first was type name with generics and where Type<T: Kek>
                        // second was generics with where <T: Kek>
                        // third was type name with generics without where Type<T>
                        let mut ctor_counter = 0;
                        let mut default_ctor_name = None;
                        for ctor in constructors {
                            let ctor_name = self.gen_ctor(&ctor, ctor_counter, &t.name);
                            if ctor.args.len() == 0 {
                                default_ctor_name = Some(ctor_name);
                            }
                            ctor_counter += 1;
                        }
                        for prop in properties {
                            self.gen_property(&prop, Some(&t.name));
                        }
                        for method in methods {
                            self.gen_method(&method, &t.name);
                        }
                        self.bindings_block.push('}');
                        self.gen_drop(&t, &class_load_name);
                        // implement Default trait for empty constructor
                        if let Some(ctor_name) = default_ctor_name {
                            self.gen_default(&t, &ctor_name);
                        }
                    }
                    TypeKind::Enum { variants } => {
                        let index_before_vis = self.bindings_block.len() - if t.vis == Visibility::Public {
                            4
                        } else {
                            11
                        }; // index to insert attributes before 'pub ' or 'pub(crate) '
                        self.bindings_block.insert_str(index_before_vis, "#[derive(Ord, PartialOrd, Hash, Eq, PartialEq, Debug, Copy, Clone)]\n");
                        self.bindings_block.push_str("enum ");
                        self.bindings_block.push_str(&get_type_name(&t, &self.naming, true));
                        self.bindings_block.push_str(" {\n");
                        for v in variants {
                            self.bindings_block.push_str(&format!("\t{} = {},\n", &v.0, &get_value(&v.1)));
                        }
                        self.bindings_block.push('}');
                    }
                    TypeKind::EnumClass { variants, methods } => {
                        let enum_load_name = self.add_load_type(&t);
                        self.bindings_block.push_str("enum ");
                        self.bindings_block.push_str(&get_type_name(&t, &self.naming, true));
                        self.bindings_block.push_str(" {\n");
                        // TODO implement variants
                        for method in methods {
                            self.gen_method(&method, &t.name);
                        }
                        self.bindings_block.push('}');
                    }
                    TypeKind::Interface { properties, methods, parents } => {
                        // TODO implement parents
                        self.bindings_block.push_str("trait ");
                        self.bindings_block.push_str(&get_type_name(&t, &self.naming, true));
                        self.bindings_block.push_str(" {\n");
                        for method in methods {
                            self.gen_method(&method, &t.name);
                        }
                        for prop in properties {
                            self.gen_property(&prop, None);
                        }
                        self.bindings_block.push('}');
                    }
                    TypeKind::Struct { constructors, properties } => {
                        // TODO implement fields
                        let struct_load_name = self.add_load_type(&t);
                        self.bindings_block.push_str(&format!(r#"struct {} {{
    ptr: Ptr
}}

impl{} {} {{
"#,
                                                              get_type_name(&t, &self.naming, true),
                                                              get_generics(&t.generics, &t.attrs, &self.naming, true),
                                                              get_type_name(&t, &self.naming, false)
                        )); // end of push_str(format!(/*..*/));
                        // What's happened in format's arguments:
                        // first - type name with generics and where Type<T: Kek>
                        // second - generics with where <T: Kek>
                        // third - type name with generics without where Type<T>
                        let mut ctor_counter = 0;
                        let mut default_ctor_name = None;
                        for ctor in constructors {
                            let ctor_name = self.gen_ctor(&ctor, ctor_counter, &t.name);
                            if ctor.args.len() == 0 {
                                default_ctor_name = Some(ctor_name);
                            }
                            ctor_counter += 1;
                        }
                        for prop in properties {
                            self.gen_property(&prop, Some(&t.name));
                        }
                        self.bindings_block.push('}');
                        self.gen_drop(&t, &struct_load_name);

                        // implement Default trait for empty constructor
                        if let Some(ctor_name) = default_ctor_name {
                            self.gen_default(&t, &ctor_name);
                        }
                    }
                    TypeKind::TypeAlias(alias) => {
                        self.bindings_block.push_str(
                            &format!("type {} = {};",
                                     get_type_name(&t, &self.naming, true),
                                     get_typeref(&alias, &self.naming)
                            ));
                    }
                }
                self.bindings_block.push('\n');
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