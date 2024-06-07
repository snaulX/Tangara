use std::path::Path;
use tangara_highlevel::*;
use crate::rust_generator::Config;
use crate::RUST_STD_LIB;

pub struct EntrypointGenerator {
    config: Config,
    package: Package,
    use_block: String,
    tgload_body: String,
    bindings_block: String,
    package_name: String
}

fn get_type_name(t: &Type) -> String {
    format!("{}_type", t.name)
}

impl EntrypointGenerator {
    pub(crate) fn new(package: Package, config: Config) -> Self {
        let package_name = format!("{}_package", package.name);
        Self {
            config,
            package,
            use_block: String::new(),
            tgload_body: String::new(),
            bindings_block: String::new(),
            package_name
        }
    }

    pub fn custom_use(mut self, use_mod: &str) -> Self {
        self.use_block.push_str(&format!("use {};\n", use_mod));
        self
    }

    fn pass_vis(&self, vis: &Visibility) -> bool {
        let vis = vis.clone();
        vis == Visibility::Public || (self.config.enable_internal && vis == Visibility::Internal)
    }

    fn get_type_name(&self, type_ref: &TypeRef) -> Option<String> {
        if let TypeRef::Name(name) = type_ref {
            Some(name.clone()) // TODO remake this so we can use not only name type references
        } else {
            None
        }
    }

    /// `this` parameter - Some: we have `self` param,
    /// `bool` inside it: is it mutable or not and `String` is name of type of `self`.
    /// Returns code for body and names of args with comma separator
    fn gen_args(&self, args: &[Argument], this: Option<(bool, String)>) -> (String, String) {
        let mut args_code = String::new();
        if this.is_some() || args.len() > 0 {
            args_code.push_str(r#"
        let args_slice = std::slice::from_raw_parts_mut(args, args_size);
        let mut args_ptr = args_slice.as_mut_ptr();"#);
        }
        if let Some((this_mut, this_type)) = this {
            let this_type_ptr = if this_mut {
                format!("*mut {}", this_type)
            } else {
                format!("*const {}", this_type)
            };
            args_code.push_str(
                &format!(r#"
        let this: {} = *(args_ptr as *mut Ptr) as {};
        args_ptr = args_ptr.add(std::mem::size_of::<{}>());"#,
                         this_type_ptr, this_type_ptr, this_type_ptr)
            );
        }
        let mut arg_names = vec![];
        for arg in args {
            let (ref_prefix, ptr_type_prefix) = match &arg.3 {
                ArgumentKind::Default => ("", "const"),
                ArgumentKind::DefaultValue(_) => ("", "const"),
                ArgumentKind::Out => ("&", "mut"),
                ArgumentKind::Ref => ("&", "mut"),
                ArgumentKind::In => ("&", "const")
            };
            let let_postfix = if ptr_type_prefix == "mut" { " mut" } else { "" };
            let arg_type = [
                ref_prefix,
                &self.get_type_name(&arg.1).unwrap_or("<ERROR TYPE GENERATOR>".to_string())
            ].concat();
            args_code.push_str(
                &format!(r#"
        let{} {}: {} = ptr::read(args_ptr as *{} {});
        args_ptr = args_ptr.add(std::mem::size_of::<{}>());"#,
                         let_postfix, arg.2, arg_type, ptr_type_prefix, arg_type, arg_type)
            );
            arg_names.push(arg.2.clone());
        }
        (args_code, arg_names.join(", "))
    }

    fn gen_dtor(&mut self, t: &Type) {
        /*let generics_ref = &t.generics.0;
        let generics = if generics_ref.len() > 0{
            format!("<{}>", generics_ref.join(", "))
        } else {
            String::new()
        };*/
        self.bindings_block.push_str(
            &format!(r#"
pub extern "C" fn {}_dtor(value: Ptr) {{
    unsafe {{
        ptr::drop_in_place(value);
        dealloc(value, Layout::new::<{}>());
    }}
}}
"#, t.name, t.name));

        self.tgload_body.push_str(
            &format!("{}.set_dtor({}_dtor);\n", get_type_name(t), t.name)
        );
    }

    fn gen_ctor(&mut self, ctor: &Constructor, t: &Type, count: usize) {
        if self.pass_vis(&ctor.vis) {
            if let Some(fn_name) = RUST_STD_LIB.get_fn_name(&ctor.attrs) {
                let ctor_name = format!("{}_ctor{}", t.name, count);
                let (args_code, arg_names) = self.gen_args(&ctor.args, None);
                let ctor_call = format!("{}::{}({})", t.name, fn_name, arg_names);
                self.bindings_block.push_str(
                    &format!(r#"
pub extern "C" fn {}(args_size: usize, args: *mut u8) -> Ptr {{
    unsafe {{{}
        let value = Box::new({});
        Box::into_raw(value) as Ptr
    }}
}}
"#, ctor_name, args_code, ctor_call));

                self.tgload_body.push_str(
                    &format!("{}.add_ctor({});\n", get_type_name(t), ctor_name)
                );
            }
            else {
                println!("[Warning] (tangara-gen::EntrypointGenerator) Bindings for constructors \
                without 'ConstructorFnName' attribute can't be generated");
            }
        }
    }

    fn gen_method(&mut self, method: &Method, t: &Type) {
        if self.pass_vis(&method.vis) {
            let this_arg = match &method.kind {
                MethodKind::Default => Some((RUST_STD_LIB.is_mutable(&method.attrs), t.name.clone())),
                MethodKind::Static => None,
                _ => {
                    return;
                }
            };
            let fn_name = format!("{}_{}", t.name, method.name);
            let (args_code, arg_names) = self.gen_args(&method.args, this_arg.clone());
            let fn_call = if this_arg.is_some() {
                format!("(*this).{}({})", method.name, arg_names)
            } else {
                format!("{}::{}({})", t.name, method.name, arg_names)
            };
            let final_code = if method.return_type.is_some() {
                format!("let to_return = Box::new({});\n\t\tBox::into_raw(to_return) as Ptr", fn_call)
            } else {
                format!("{};\n\t\tptr::null_mut()", fn_call)
            };
            self.bindings_block.push_str(
                &format!(r#"
pub extern "C" fn {}(args_size: usize, args: *mut u8) -> Ptr {{
    unsafe {{{}
        {}
    }}
}}
"#, fn_name, args_code, final_code));

            self.tgload_body.push_str(
                &format!("{}.add_method({}, {});\n", get_type_name(t), method.id, fn_name)
            );
        }
    }

    fn gen_property(&mut self, prop: &Property, t: &Type) {
        if self.pass_vis(&prop.getter_visibility) {
            let getter_name = format!("{}_get_{}", t.name, prop.name);
            self.bindings_block.push_str(
                &format!(r#"
pub extern "C" fn {0}(this: Ptr) -> Ptr {{
    unsafe {{
        let this: *const {1} = this as *const {1};
        let to_return = Box::new((*this).get_{2}());
        Box::into_raw(to_return) as Ptr
    }}
}}
"#, getter_name, t.name, prop.name));

            let setter = if let Some(setter_vis) = prop.setter_visibility {
                if self.pass_vis(&setter_vis) {
                    let setter_name = format!("{}_set_{}", t.name, prop.name);
                    let prop_type = self.get_type_name(&prop.prop_type)
                        .unwrap_or("<ERROR TYPE GENERATOR>".to_string());
                    self.bindings_block.push_str(
                        &format!(r#"
pub extern "C" fn {0}(this: Ptr, object: Ptr) {{
    unsafe {{
        let this: *mut {1} = this as *mut {1};
        let {2}: {3} = ptr::read(object as *const {3});
        (*this).set_{2}({2});
    }}
}}
"#, setter_name, t.name, prop.name, prop_type));
                    format!("Some({})", setter_name)
                }
                else {
                    "None".to_string()
                }
            } else {
                "None".to_string()
            };

            self.tgload_body.push_str(
                &format!("{}.add_property({}, Property {{ getter: {}, setter: {} }});\n",
                         get_type_name(t), prop.id, getter_name, setter)
            );
        }
    }

    fn gen_field(&mut self, field: &Field, t: &Type) {
        if self.pass_vis(&field.vis) {
            let getter_name = format!("{}_get_{}", t.name, field.name);
            self.bindings_block.push_str(
                &format!(r#"
pub extern "C" fn {0}(this: Ptr) -> Ptr {{
    unsafe {{
        let this: *const {1} = this as *const {1};
        let to_return = Box::new((*this).{2});
        Box::into_raw(to_return) as Ptr
    }}
}}
"#, getter_name, t.name, field.name));

                let setter_name = format!("{}_set_{}", t.name, field.name);
                let field_type = self.get_type_name(&field.field_type)
                    .unwrap_or("<ERROR TYPE GENERATOR>".to_string());
                self.bindings_block.push_str(
                    &format!(r#"
pub extern "C" fn {0}(this: Ptr, object: Ptr) {{
    unsafe {{
        let this: *mut {1} = this as *mut {1};
        let {2}: {3} = ptr::read(object as *const {3});
        (*this).{2} = {2};
    }}
}}
"#, setter_name, t.name, field.name, field_type));

            self.tgload_body.push_str(
                &format!("{}.add_property({}, Property {{ getter: {}, setter: {} }});\n",
                         get_type_name(t), field.id, getter_name, setter_name)
            );
        }
    }

    fn gen_static_property(&mut self, prop: &Property, t: &Type) {
        if self.pass_vis(&prop.getter_visibility) {
            let getter_name = format!("{}_get_static_{}", t.name, prop.name);
            self.bindings_block.push_str(
                &format!(r#"
pub extern "C" fn {0}() -> Ptr {{
    unsafe {{
        let to_return = Box::new({1}::get_{2}());
        Box::into_raw(to_return) as Ptr
    }}
}}
"#, getter_name, t.name, prop.name));

            let setter = if let Some(setter_vis) = prop.setter_visibility {
                if self.pass_vis(&setter_vis) {
                    let setter_name = format!("{}_set_static_{}", t.name, prop.name);
                    let prop_type = self.get_type_name(&prop.prop_type)
                        .unwrap_or("<ERROR TYPE GENERATOR>".to_string());
                    self.bindings_block.push_str(
                        &format!(r#"
pub extern "C" fn {0}(object: Ptr) {{
    unsafe {{
        let {2}: {3} = ptr::read(object as *const {3});
        {1}::set_{2}({2});
    }}
}}
"#, setter_name, t.name, prop.name, prop_type));
                    format!("Some({})", setter_name)
                }
                else {
                    "None".to_string()
                }
            } else {
                "None".to_string()
            };

            self.tgload_body.push_str(
                &format!("{}.add_static({}, StaticProperty {{ getter: {}, setter: {} }});\n",
                         get_type_name(t), prop.id, getter_name, setter)
            );
        }
    }

    // TODO
    fn gen_static_field(&mut self, field: &Field, t: &Type) {
        if self.pass_vis(&field.vis) {
            let getter_name = format!("{}_get_static_{}", t.name, field.name);
            self.bindings_block.push_str(
                &format!(r#"
pub extern "C" fn {}() -> Ptr {{
    unsafe {{
        let to_return = Box::new({}::{});
        Box::into_raw(to_return) as Ptr
    }}
}}
"#, getter_name, t.name, field.name));

                let setter_name = format!("{}_set_static_{}", t.name, field.name);
                let field_type = self.get_type_name(&field.field_type)
                    .unwrap_or("<ERROR TYPE GENERATOR>".to_string());
                self.bindings_block.push_str(
                    &format!(r#"
pub extern "C" fn {0}(object: Ptr) {{
    unsafe {{
        let {2}: {3} = ptr::read(object as *const {3});
        {1}::{2} = {2};
    }}
}}
"#, setter_name, t.name, field.name, field_type));

            self.tgload_body.push_str(
                &format!("{}.add_static({}, StaticProperty {{ getter: {}, setter: {} }});\n",
                         get_type_name(t), field.id, getter_name, setter_name)
            );
        }
    }

    fn gen_variant(&mut self, variant: &Variant, t: &Type) {
        if self.pass_vis(&variant.vis) {
            let fn_name = format!("{}_{}", t.name, variant.name);

            // Translate properties into arguments
            let mut args = vec![];
            for v_field in &variant.fields {
                args.push(Argument::from(v_field.clone()))
            }

            let (enum_variant, args_code) = if args.len() > 0 {
                let (args_code, arg_names) = self.gen_args(&args, None);
                if RUST_STD_LIB.is_tuple_variant(&variant.attrs) {
                    (format!("{}::{}({})", t.name, variant.name, arg_names), args_code)
                } else {
                    (format!("{}::{} {{ {} }}", t.name, variant.name, arg_names), args_code)
                }
            } else {
                (format!("{}::{}", t.name, variant.name), String::new())
            };
            let final_code = format!("let return_value = Box::new({});\n\t\tBox::into_raw(return_value) as Ptr",
                                     enum_variant);
            self.bindings_block.push_str(
                &format!(r#"
pub extern "C" fn {}(args_size: usize, args: *mut u8) -> Ptr {{
    unsafe {{{}
        {}
    }}
}}
"#, fn_name, args_code, final_code));

            self.tgload_body.push_str(
                &format!("{}.add_method({}, {});\n", get_type_name(t), variant.id, fn_name)
            );
        }
    }

    fn generate(&mut self) {
        self.tgload_body.push_str(
            &format!("let mut {} = ctx.add_package({});\n", self.package_name, self.package.id)
        );
        let types = self.package.types.to_vec();
        for t in types {
            if t.generics.0.len() > 0 {
                // while we cannot handle generics so skip it
                println!("[Warning] Skip {} type because EntrypointGenerator cannot handle generics", t.name);
                continue;
            }
            if self.pass_vis(&t.vis) {
                match &t.kind {
                    TypeKind::Class {
                        is_sealed,
                        constructors,
                        properties,
                        fields,
                        static_properties,
                        static_fields,
                        methods,
                        parents
                    } => {
                        let type_name = get_type_name(&t);
                        self.tgload_body.push_str(
                            &format!("let mut {} = {}.add_type({});\n", type_name, self.package_name, t.id)
                        );
                        self.gen_dtor(&t);
                        let mut count = 0usize;
                        for ctor in constructors {
                            self.gen_ctor(ctor, &t, count);
                            count += 1;
                        }
                        for prop in properties {
                            self.gen_property(prop, &t);
                        }
                        for static_prop in static_properties {
                            self.gen_property(static_prop, &t); // TODO implement static properties
                        }
                        for field in fields {
                            self.gen_field(field, &t);
                        }
                        for static_field in static_fields {
                            self.gen_field(static_field, &t); // TODO implement static field
                        }
                        for method in methods {
                            self.gen_method(method, &t);
                        }
                    }
                    TypeKind::EnumClass {
                        variants,
                        methods
                    } => {
                        let type_name = get_type_name(&t);
                        self.tgload_body.push_str(
                            &format!("let mut {} = {}.add_type({});\n", type_name, self.package_name, t.id)
                        );
                        self.gen_dtor(&t);
                        for variant in variants {
                            self.gen_variant(variant, &t);
                        }
                        for method in methods {
                            self.gen_method(method, &t);
                        }
                    }
                    TypeKind::Struct {
                        constructors,
                        fields,
                        static_fields
                    } => {
                        let type_name = get_type_name(&t);
                        self.tgload_body.push_str(
                            &format!("let mut {} = {}.add_type({});\n", type_name, self.package_name, t.id)
                        );
                        self.gen_dtor(&t);
                        let mut count = 0usize;
                        for ctor in constructors {
                            self.gen_ctor(ctor, &t, count);
                            count += 1;
                        }
                        for field in fields {
                            self.gen_field(field, &t);
                        }
                        for static_field in static_fields {
                            self.gen_field(static_field, &t); // TODO implement static field
                        }
                    }
                    _ => {
                        // skip
                    }
                }
            }
        }
    }

    pub fn write_to<P: AsRef<Path>>(mut self, path: P) -> std::io::Result<()> {
        self.generate();
        let disclaimer = r#"// This file was generated by tangara-gen
// All changes in this file will discard after rebuilding project
use std::ptr;
use std::alloc::{dealloc, Layout};
use tangara::context::{Context, Ptr, Property};
"#.to_string();
        let mut tgload_body = self.tgload_body.replace("\n", "\n\t");
        tgload_body.remove(tgload_body.len() - 1); // remove last extra '\t'
        let tgload = format!("#[no_mangle]\npub extern \"C\" fn {}(ctx: &mut Context) {{\n\t{}}}\n",
                             self.config.load_name, tgload_body);
        std::fs::write(path, String::from_iter([disclaimer, self.use_block, self.bindings_block, tgload]))
    }
}