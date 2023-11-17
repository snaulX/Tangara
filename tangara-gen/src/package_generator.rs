use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use proc_macro2::Ident;
use quote::ToTokens;
use syn::{Item, parse_file, Visibility};
use tangara_highlevel::builder::*;
use tangara_highlevel::{Package, Visibility as TgVis};

pub struct Config {
    pub ctor_names: Vec<String>,
    pub generate_pub_fields: bool
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ctor_names: vec!["new".to_string()],
            generate_pub_fields: true
        }
    }
}

pub struct PackageGenerator {
    config: Config,
    package_builder: Rc<RefCell<PackageBuilder>>,
    structs: HashMap<String, ClassBuilder>
}

impl PackageGenerator {
    pub fn new(package_name: &str, config: Config) -> Self {
        Self {
            config,
            package_builder: PackageBuilder::new(package_name),
            structs: HashMap::new()
        }
    }

    fn get_or_create_struct(&mut self, name: &Ident) -> &mut ClassBuilder {
        let name = name.to_string();
        self.structs.entry(name.clone()).or_insert(create_class(self.package_builder.clone(), &name))
    }

    fn parse_item(&mut self, item: &Item) {
        match item {
            Item::Enum(_) => {}
            Item::Impl(_) => {}
            Item::Mod(mod_item) => {
                let prev_ns = self.package_builder.borrow().get_namespace();
                let next_ns = mod_item.ident.to_string().replace("::", ".");
                let mut new_ns = String::with_capacity(prev_ns.len() + 1 + next_ns.len());
                new_ns.push_str(&prev_ns);
                new_ns.push('.');
                new_ns.push_str(&next_ns);
                self.package_builder.borrow_mut().set_namespace(&new_ns);
                if let Some((_, items)) = &mod_item.content {
                    for it in items {
                        self.parse_item(it);
                    }
                }
                self.package_builder.borrow_mut().set_namespace(&prev_ns);
            }
            Item::Struct(struct_item) => {
                let class_builder = self.get_or_create_struct(&struct_item.ident);
                class_builder.set_visibility(match &struct_item.vis {
                    Visibility::Public(_) => TgVis::Public,
                    Visibility::Restricted(sub_vis) => {
                        let sub_vis_name = sub_vis.path.to_token_stream().to_string();
                        if sub_vis_name == "super" {
                            TgVis::Protected
                        } else {
                            TgVis::Internal
                        }
                    }
                    Visibility::Inherited => TgVis::Private
                });
            }
            Item::Trait(_) => {}
            Item::Type(_) => {}
            Item::Union(_) => {}
            _ => {}
        }
    }

    pub fn parse_code(mut self, code: &str) -> Self {
        let syntax_tree = parse_file(code).expect("Failed to parse Rust code");

        for item in syntax_tree.items {
            self.parse_item(&item);
        }

        self
    }

    pub fn parse_file<P: AsRef<Path>>(self, path: P) -> Self {
        let rust_code = std::fs::read_to_string(path).expect("Failed to read file");
        self.parse_code(&rust_code)
    }

    pub fn generate(self) -> Package {
        for (_, cb) in self.structs {
            cb.build();
        }
        self.package_builder.borrow().build()
    }

    pub fn generate_to_file<P: AsRef<Path>>(self, path: P) -> std::io::Result<()> {
        std::fs::write(path, format!("{:?}", self.generate()))
    }
}