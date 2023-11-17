use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use proc_macro2::Ident;
use quote::ToTokens;
use syn::{Item, parse_file, ReturnType, TraitItem, Type, Visibility};
use tangara_highlevel::builder::*;
use tangara_highlevel::{Package, TypeRef, Visibility as TgVis};

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

fn get_visibility(vis: &Visibility) -> TgVis {
    match vis {
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
    }
}

fn get_typeref(t: &Type) -> Option<TypeRef> {
    match t {
        Type::Array(_) => None,
        Type::BareFn(_) => None,
        Type::Group(_) => None,
        Type::ImplTrait(_) => None,
        Type::Macro(_) => None,
        Type::Never(_) => None,
        Type::Paren(paren_type) => {
            get_typeref(&paren_type.elem)
        },
        Type::Path(path_type) => {
            let mut path = String::new();
            for seg in &path_type.path.segments {
                path.push_str(&seg.ident.to_string());
                path.push('.');
            }
            path.remove(path.len() - 1); // remove last '.'
            Some(TypeRef::Name(path))
        },
        Type::Ptr(ptr_type) => {
            // TODO add attribute of mutability if ref_type is mutable
            // TODO change type from default to Ptr<T>
            get_typeref(&ptr_type.elem)
        },
        Type::Reference(ref_type) => {
            // TODO add attribute of mutability if ref_type is mutable
            get_typeref(&ref_type.elem)
        },
        Type::Slice(_) => None,
        Type::TraitObject(_) => None,
        Type::Tuple(tuple_type) => {
            let mut types = vec![];
            for tt in &tuple_type.elems {
                let ott = get_typeref(tt);
                if ott.is_some() {
                    types.push(ott.unwrap());
                }
            }
            Some(TypeRef::Tuple(types))
        }
        _ => None
    }
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
                let mut class_builder = self.get_or_create_struct(&struct_item.ident);
                class_builder.set_visibility(get_visibility(&struct_item.vis));
            }
            Item::Trait(trait_item) => {
                let mut interface_builder = create_interface(
                    self.package_builder.clone(),
                    &trait_item.ident.to_string() // name
                );
                interface_builder.set_visibility(get_visibility(&trait_item.vis));
                for it in &trait_item.items {
                    match it {
                        TraitItem::Const(_) => {}
                        TraitItem::Fn(fn_item) => {
                            let mut fn_builder = interface_builder.add_method(&fn_item.sig.ident.to_string());

                            // Parse return type
                            match &fn_item.sig.output {
                                ReturnType::Default => {} // return type of fn_builder by default is nothing
                                ReturnType::Type(_, ret_type) => {
                                    if let Some(ret_typeref) = get_typeref(ret_type) {
                                        /*if let TypeRef::Tuple(types) = &ret_typeref {
                                            if types.len() != 0 {
                                                fn_builder.return_type(ret_typeref);
                                            }
                                        }
                                        else {*/
                                            fn_builder.return_type(ret_typeref);
                                        //}
                                    }
                                }
                            }

                            // Parse arguments

                            fn_builder.build();
                        }
                        TraitItem::Type(_) => {}
                        TraitItem::Macro(_) => {}
                        _ => {}
                    }
                }
                interface_builder.build();
            }
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