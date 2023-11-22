use std::path::Path;
use tangara_highlevel::{Package, Property, Type, TypeKind, TypeRef};
use tangara_highlevel::Visibility as TgVis;
use crate::RUST_STD_LIB;

pub struct Config {
    /// Enable generation of internal types and members.
    /// Default value: `false`
    pub enable_internal: bool
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enable_internal: false
        }
    }
}

pub struct RustGenerator {
    config: Config,
    package: Package
}

impl RustGenerator {
    pub fn new(package: Package, config: Config) -> Self {
        Self {
            config,
            package
        }
    }

    pub fn generate_entrypoint(mut self) -> EntrypointGenerator {
        EntrypointGenerator::new(self.package, self.config)
    }
}

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

    fn pass_vis(&self, vis: &TgVis) -> bool {
        let vis = vis.clone();
        vis == TgVis::Public || (self.config.enable_internal && vis == TgVis::Internal)
    }

    fn gen_dtor(&mut self, t: &Type) {
        let generics_ref = &t.generics.0;
        let generics = if generics_ref.len() > 0{
            format!("<{}>", generics_ref.join(", "))
        } else {
            String::new()
        };
        self.bindings_block.push_str(
            &format!(r#"
pub extern "C" fn {}_dtor(value: Ptr) {{
    unsafe {{
        ptr::drop_in_place(value);
        dealloc(value, Layout::new::<{}{}>());
    }}
}}
"#, t.name, t.name, generics));

        self.tgload_body.push_str(
            &format!("{}.set_dtor({}_dtor);\n", get_type_name(t), t.name)
        );
    }

    fn gen_property(&mut self, prop: &Property, t: &Type) {
        if self.pass_vis(&prop.getter_visibility) {
            let is_field = RUST_STD_LIB.is_struct_field(&prop.attrs);
            let get_code = if is_field {
                prop.name.clone()
            } else {
                format!("get_{}()", prop.name)
            };
            let getter_name = format!("{}_get_{}", t.name, prop.name);
            self.bindings_block.push_str(
                &format!(r#"
pub extern "C" fn {}(this: Ptr) -> Ptr {{
    unsafe {{
        let this: *const {} = this as *const {};
        let to_return = Box::new((*this).{});
        Box::into_raw(to_return) as Ptr
    }}
}}
"#, getter_name, t.name, t.name, get_code));

            let setter = if let Some(setter_vis) = prop.setter_visibility {
                if self.pass_vis(&setter_vis) {
                    let set_code = if is_field {
                        format!("{} = {}", prop.name, prop.name)
                    } else {
                        format!("set_{}({})", prop.name, prop.name)
                    };
                    let setter_name = format!("{}_set_{}", t.name, prop.name);
                    let prop_type = if let TypeRef::Name(name) = &prop.prop_type {
                        name // TODO remake this so we can use not only name type references
                    } else {
                        "<ERROR TYPE GENERATOR>"
                    };
                    self.bindings_block.push_str(
                        &format!(r#"
pub extern "C" fn {}(this: Ptr, object: Ptr) {{
    unsafe {{
        let this: *mut {} = this as *mut {};
        let {}: {} = ptr::read(object as *const {});
        (*this).{};
    }}
}}
"#, setter_name, t.name, t.name, prop.name, prop_type, prop_type, set_code));
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

    fn generate(&mut self) {
        self.tgload_body.push_str(
            &format!("let mut {} = ctx.add_package({});\n", self.package_name, self.package.id)
        );
        let types = self.package.types.to_vec();
        for t in types {
            if t.generics.0.len() > 0 {
                // while we cannot handle generics so skip it
                continue;
            }
            if self.pass_vis(&t.vis) {
                match &t.kind {
                    TypeKind::Class(_, ctors, props, methods, _) => {
                        let type_name = get_type_name(&t);
                        self.tgload_body.push_str(
                            &format!("let mut {} = {}.add_type({});\n", type_name, self.package_name, t.id)
                        );
                        self.gen_dtor(&t);
                        for prop in props {
                            self.gen_property(prop, &t);
                        }
                    }
                    TypeKind::EnumClass(variants, methods) => {}
                    TypeKind::Struct(ctors, props) => {
                        let type_name = get_type_name(&t);
                        self.tgload_body.push_str(
                            &format!("let mut {} = {}.add_type({});\n", type_name, self.package_name, t.id)
                        );
                        self.gen_dtor(&t);
                        for prop in props {
                            self.gen_property(prop, &t);
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
        let tgload = format!("#[no_mangle]\npub extern \"C\" fn tgLoad(ctx: &mut Context) {{\n\t{tgload_body}}}\n");
        std::fs::write(path, String::from_iter([disclaimer, self.use_block, self.bindings_block, tgload]))
    }
}