use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;
use quote::ToTokens;
use syn::*;
use syn::punctuated::Punctuated;
use tangara_highlevel::builder::*;
use tangara_highlevel::{Attribute, Package, TypeKind, TypeRef, Value, Visibility as TgVis};

pub struct Config {
    /// Names of traits which we **don't** need inherit from
    ///
    /// Default: `"Default", "From"`
    pub dont_inherit_traits: Vec<String>,
    /// Function names that implemented as constructors.
    ///
    /// For example: if we have there name `new`, `MyStruct::new(args)` will added to type as constructor.
    /// Default: `"new"`
    pub ctor_names: Vec<String>,
    /// Generate properties from public fields
    /// Default: `true`
    pub generate_pub_fields: bool,
    /// Generate properties from get_, set_ pair methods
    /// Default: `true`
    pub generate_properties: bool
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dont_inherit_traits: vec!["Default".to_string(), "From".to_string()],
            ctor_names: vec!["new".to_string()],
            generate_pub_fields: true,
            generate_properties: true
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

// TODO also return attributes (we can make ones to mark is as mutable, reference and etc.)
fn get_typeref(t: &Type) -> Option<TypeRef> {
    match t {
        Type::Array(_) => Some(TypeRef::from("Array")),
        Type::BareFn(fn_type) => {
            // Parse return type
            let return_type = match &fn_type.output {
                ReturnType::Default => None,
                ReturnType::Type(_, ret_type) => {
                    if let Some(ret_typeref) = get_typeref(ret_type) {
                        Some(Box::new(ret_typeref))
                    }
                    else {
                        None
                    }
                }
            };

            // Parse arguments
            let mut args = vec![];
            for input in &fn_type.inputs {
                args.push(get_typeref(&input.ty).expect("Argument type cannot be None"));
            }

            Some(TypeRef::Fn(return_type, args))
        },
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
        Type::Slice(_) => Some(TypeRef::from("Array")),
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

fn parse_return_type<T: MethodCollector>(fn_builder: &mut MethodBuilder<T>, return_type: &ReturnType) {
    match return_type {
        ReturnType::Default => {} // return type of fn_builder by default is nothing
        ReturnType::Type(_, ret_type) => {
            if let Some(ret_typeref) = get_typeref(ret_type) {
                fn_builder.return_type(ret_typeref);
            }
        }
    }
}

fn parse_arg<T: MethodCollector>(fn_builder: &mut MethodBuilder<T>, fn_arg: &PatType) {
    if let Pat::Ident(arg_ident) = &fn_arg.pat.deref() {
        let arg_name = arg_ident.ident.to_string();
        let arg_type = get_typeref(&fn_arg.ty).expect("Arg type cannot be None");
        if arg_ident.mutability.is_some() {
            fn_builder.arg_ref(arg_type, arg_name.as_str());
        }
        else {
            fn_builder.arg(arg_type, arg_name.as_str());
        }
    }
    else {
        panic!("Function arg name is not ident");
    }
}

fn parse_generics<T: GenericsCollector + AttributeCollector>(builder: &mut T, generics: &Generics) {
    let mut generic_types = vec![];
    let mut generic_wheres = vec![];

    // Local function for parsing generics bounds
    let mut parse_bounds = |builder: &mut T, bounded: String, bounds: &Punctuated<TypeParamBound, Token![+]>| {
        for bound in bounds {
            match bound {
                TypeParamBound::Trait(trait_bound) => {
                    let mut trait_name = String::new();
                    for seg in &trait_bound.path.segments {
                        trait_name.push_str(&seg.ident.to_string());
                        match &seg.arguments {
                            PathArguments::None => {}
                            PathArguments::AngleBracketed(angle) => {
                                trait_name.push('<');
                                for ga in &angle.args {
                                    match &ga {
                                        GenericArgument::Type(gt) => {
                                            trait_name.push_str(&gt.to_token_stream().to_string());
                                            trait_name.push(',');
                                        }
                                        _ => {
                                            println!("[Warning] (tangara-gen::PackageGenerator) \
                                                    Recursive generic arguments are not supported.");
                                        }
                                    }
                                }
                                if angle.args.len() > 0 {
                                    // Remove last extra ','
                                    trait_name.remove(trait_name.len() - 1);
                                }
                                trait_name.push('>');
                            }
                            PathArguments::Parenthesized(_) => {
                                println!("[Warning] (tangara-gen::PackageGenerator) \
                                        Parenthesized generic arguments are not supported.");
                            }
                        }
                        trait_name.push('.');
                    }
                    // Remove last extra '.'
                    trait_name.remove(trait_name.len() - 1); // I hope segments wasn't empty...
                    // Finally add bound
                    generic_wheres.push((bounded.clone(), TypeRef::Name(trait_name)));
                }
                TypeParamBound::Lifetime(lifetime) => {
                    // Add attribute to mark for Tangara that in Rust this generic has lifetime
                    let lt = lifetime.ident.to_string();
                    builder.add_attribute(
                        Attribute(
                            TypeRef::from("Tangara.Rust.LifetimeGeneric"),
                            vec![Value::String(bounded.clone()), Value::String(lt)]
                        )
                    );
                }
                _ => {}
            }
        }
    };

    for gp in &generics.params {
        match gp {
            GenericParam::Type(generic_type) => {
                let generic_name = generic_type.ident.to_string();
                generic_types.push(generic_name.clone());
                parse_bounds(builder, generic_name, &generic_type.bounds);
            }
            GenericParam::Lifetime(lifetime) => {
                // Add attribute to mark for Tangara that in Rust it has lifetime
                let lt = lifetime.lifetime.ident.to_string();
                builder.add_attribute(
                    Attribute(TypeRef::from("Tangara.Rust.Lifetime"), vec![Value::String(lt)])
                );
            }
            GenericParam::Const(_) => {
                println!("[Warning] (tangara-gen::PackageGenerator) Const are not supported in generics.");
            }
        }
    }

    if let Some(where_clause) = &generics.where_clause {
        for predicate in &where_clause.predicates {
            match predicate {
                WherePredicate::Lifetime(_) => {}
                WherePredicate::Type(type_predicate) => {
                    if let Some(TypeRef::Name(type_name)) = get_typeref(&type_predicate.bounded_ty) {
                        parse_bounds(builder, type_name, &type_predicate.bounds);
                    } else {
                        panic!("Can't get TypeRef::Name from bounded type in 'where' clause");
                    }
                }
                _ => {}
            }
        }
    }

    builder.generics(generic_types);
    builder.generic_wheres(generic_wheres);
}

impl PackageGenerator {
    pub fn new(package_name: &str, config: Config) -> Self {
        Self {
            config,
            package_builder: PackageBuilder::new(package_name),
            structs: HashMap::new()
        }
    }

    fn get_or_create_struct(&mut self, name: String) -> &mut ClassBuilder {
        self.structs.entry(name.clone()).or_insert(create_class(self.package_builder.clone(), &name))
    }

    fn parse_item(&mut self, item: &Item) {
        match item {
            Item::Enum(_) => {
                // TODO implement
            }
            Item::Impl(impl_item) => {
                let mut for_name = None;
                // Check situation on 'impl Trait for Struct'
                if let Some((_, type_name, _)) = &impl_item.trait_ {
                    for_name = Some(type_name.to_token_stream().to_string());
                }
                if let TypeRef::Name(type_name) = get_typeref(&impl_item.self_ty)
                    .expect("Type in 'impl' cannot be None") {
                    let ctor_names = self.config.ctor_names.to_vec();
                    let dont_inherit_traits = self.config.dont_inherit_traits.to_vec();

                    let cb = self.get_or_create_struct(type_name.clone());
                    if let Some(trait_name) = for_name {
                        // Again, if impl is with trait, then we need to inherit class from it
                        // But if really needs to. Because some traits is not important to inherit from.
                        if !dont_inherit_traits.contains(&trait_name) {
                            cb.inherits(TypeRef::Name(trait_name));
                        }
                    }
                    for item_impl in &impl_item.items {
                        match item_impl {
                            ImplItem::Fn(fn_item) => {
                                // TODO check on get_ set_ pair functions to generate properties
                                let fn_sig = &fn_item.sig;
                                let name = fn_sig.ident.to_string();
                                // Check on constructor name
                                if ctor_names.contains(&name) {
                                    // Make constructor
                                    let mut ctor_builder = cb.add_constructor();
                                    ctor_builder.set_visibility(get_visibility(&fn_item.vis));
                                    // Add attribute: name of 'fn' associated to this constructor
                                    ctor_builder.add_attribute(Attribute(
                                        TypeRef::from("Tangara.Rust.Metadata.ConstructorFnName"),
                                        vec![Value::String(name.clone())]
                                    ));

                                    // Check for generics emptiness
                                    if fn_sig.generics.params.len() > 0 {
                                        panic!("Constructor can't have generics");
                                    }

                                    // Check for correct return type
                                    match &fn_sig.output {
                                        ReturnType::Default => {
                                            panic!("Constructor can't return nothing");
                                        }
                                        ReturnType::Type(_, return_type_boxed) => {
                                            let return_type = return_type_boxed.to_token_stream().to_string();
                                            if return_type != "Self" &&
                                                return_type != type_name {
                                                panic!("Return type of constructor can't be not as type of impl: {} != {}",
                                                       return_type, type_name);
                                            }
                                        }
                                    }

                                    // Parse arguments
                                    for arg in &fn_sig.inputs {
                                        match arg {
                                            FnArg::Receiver(_) => {
                                                panic!("Constructor can't contains 'self' argument");
                                            }
                                            FnArg::Typed(ctor_arg) => {
                                                if let Pat::Ident(arg_ident) = &ctor_arg.pat.deref() {
                                                    let arg_name = arg_ident.ident.to_string();
                                                    let arg_type = get_typeref(&ctor_arg.ty).expect("Arg type cannot be None");
                                                    if arg_ident.mutability.is_some() {
                                                        ctor_builder.arg_ref(arg_type, arg_name.as_str());
                                                    }
                                                    else {
                                                        ctor_builder.arg(arg_type, arg_name.as_str());
                                                    }
                                                }
                                                else {
                                                    panic!("Constructor arg name is not ident");
                                                }
                                            }
                                        }
                                    }

                                    ctor_builder.build();
                                }
                                else {
                                    // Make function
                                    let mut fn_builder = cb.add_method(&name);
                                    fn_builder.set_visibility(get_visibility(&fn_item.vis));
                                    parse_generics(&mut fn_builder, &fn_item.sig.generics);
                                    parse_return_type(&mut fn_builder, &fn_sig.output);

                                    // Parse arguments
                                    //let mut have_self = false;
                                    for arg in &fn_sig.inputs {
                                        match arg {
                                            FnArg::Receiver(_) => {
                                                //have_self = true;
                                            }
                                            FnArg::Typed(fn_arg) => {
                                                parse_arg(&mut fn_builder, fn_arg);
                                            }
                                        }
                                    }
                                    // TODO make function be static if haven't 'self' argument

                                    fn_builder.build();
                                }
                            }
                            ImplItem::Type(_) => {} // TODO add checks in typeref making in function (return or args) on this type
                            _ => {}
                        }
                    }
                } // if let TypeRef::Name(type_name) = get_typeref(&impl_item.self_ty)
                else {
                    panic!("TypeRef from 'impl' root must be Name");
                }
            }
            Item::Mod(mod_item) => {
                let prev_ns = self.package_builder.borrow().get_namespace();
                let next_ns = mod_item.ident.to_string().replace("::", ".");
                let mut new_ns = String::with_capacity(prev_ns.len() + 1 + next_ns.len());
                new_ns.push_str(&prev_ns);
                new_ns.push('.');
                new_ns.push_str(&next_ns);
                self.package_builder.borrow_mut().set_namespace(&new_ns);
                // Set default type visibility to mod's
                let old_vis = self.package_builder.borrow().type_visibility;
                self.package_builder.borrow_mut().type_visibility = get_visibility(&mod_item.vis);
                if let Some((_, items)) = &mod_item.content {
                    for it in items {
                        self.parse_item(it);
                    }
                }
                let mut builder = self.package_builder.borrow_mut();
                builder.set_namespace(&prev_ns);
                builder.type_visibility = old_vis;
            }
            Item::Struct(struct_item) => {
                let generate_pub_fields = self.config.generate_pub_fields;
                let class_builder = self.get_or_create_struct(struct_item.ident.to_string());
                class_builder.set_visibility(get_visibility(&struct_item.vis));
                parse_generics(class_builder, &struct_item.generics);

                if generate_pub_fields {
                    for field in &struct_item.fields {
                        if let Visibility::Public(_) = field.vis {
                            if let Some(field_ident) = &field.ident {
                                let mut prop_builder = class_builder.add_property(
                                    get_typeref(&field.ty).expect("Field cannot have type None"),
                                    &field_ident.to_string()
                                );
                                prop_builder.getter_visibility(TgVis::Public);
                                prop_builder.setter_visibility(TgVis::Public);
                                prop_builder.build();
                            }
                        }
                    }
                }
            }
            Item::Trait(trait_item) => {
                let mut interface_builder = create_interface(
                    self.package_builder.clone(),
                    &trait_item.ident.to_string() // name
                );
                interface_builder.set_visibility(get_visibility(&trait_item.vis));
                parse_generics(&mut interface_builder, &trait_item.generics);

                for it in &trait_item.items {
                    match it {
                        TraitItem::Fn(fn_item) => {
                            // TODO check on get_ set_ pair functions to generate properties
                            let mut fn_builder = interface_builder.add_method(&fn_item.sig.ident.to_string());
                            fn_builder.set_visibility(TgVis::Public);
                            parse_generics(&mut fn_builder, &fn_item.sig.generics);
                            parse_return_type(&mut fn_builder, &fn_item.sig.output);

                            // Parse arguments
                            let mut have_self = false;
                            for arg in &fn_item.sig.inputs {
                                match arg {
                                    FnArg::Receiver(_) => {
                                        have_self = true;
                                    }
                                    FnArg::Typed(fn_arg) => {
                                        parse_arg(&mut fn_builder, fn_arg);
                                    }
                                }
                            }
                            if !have_self {
                                println!("[Warning] (tangara-gen::PackageGenerator) Trait \
                                 (interface) method must have 'self' argument. Ignoring it.");
                            }

                            fn_builder.build();
                        }
                        TraitItem::Type(_) => {} // TODO add checks in typeref making in function (return or args) on this type
                        _ => {}
                    }
                }

                interface_builder.build();
            }
            Item::Type(type_item) => {
                let mut alias_builder = create_alias(
                    self.package_builder.clone(),
                    &type_item.ident.to_string(),
                    get_typeref(&type_item.ty).expect("Type in alias cannot be None")
                );
                alias_builder.set_visibility(get_visibility(&type_item.vis));
                parse_generics(&mut alias_builder, &type_item.generics);
                alias_builder.build();
            }
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
            let result = cb.get_type();
            if let TypeKind::Class(_, ctors, props, methods, parents) = &result.kind {
                let mut builder = self.package_builder.borrow_mut();
                builder.add_type(
                    // Change type's kind on Struct if it's possible
                    if methods.len() == 0 && parents.len() == 0 {
                        let mut result = result.clone();
                        result.kind = TypeKind::Struct(ctors.to_vec(), props.to_vec());
                        result
                    }
                    else {
                        result.clone()
                    }
                );
            }
        }
        let mut builder = self.package_builder.borrow_mut();
        builder.add_attribute(Attribute(TypeRef::from("Tangara.Metadata.Lang"), vec![Value::from("Rust")]));
        builder.build()
    }
}