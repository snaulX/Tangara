use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;
use syn::*;
use syn::punctuated::Punctuated;
use tangara_highlevel::builder::*;
use tangara_highlevel::{Attribute, MethodKind, NamingConventions, Package, TypeKind, TypeRef, Value, Visibility as TgVis};
use crate::RUST_STD_LIB;

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
    /// Generate properties from get_, set_ pair methods
    /// Default: `true`
    pub generate_properties: bool
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dont_inherit_traits: vec!["Default".to_string(), "From".to_string()],
            ctor_names: vec!["new".to_string()],
            generate_properties: true
        }
    }
}

pub struct PackageGenerator {
    config: Config,
    package_builder: Rc<RefCell<PackageBuilder>>,
    structs: HashMap<String, ClassBuilder>
}

fn get_from_path(syn_path: &syn::Path) -> TypeRef {
    let mut path = String::new();
    let mut generics = vec![];
    for seg in &syn_path.segments {
        path.push_str(&seg.ident.to_string());
        match &seg.arguments {
            PathArguments::None => {}
            PathArguments::AngleBracketed(angle) => {
                for ga in &angle.args {
                    match &ga {
                        GenericArgument::Type(gt) => {
                            generics.push(get_typeref(gt).expect("Generic type can't be None").0);
                        }
                        _ => {
                            println!("[Warning] (tangara-gen::PackageGenerator) \
                                                    other (then type) generic arguments are not supported in path.");
                        }
                    }
                }
            }
            PathArguments::Parenthesized(_) => {
                println!("[Warning] (tangara-gen::PackageGenerator) \
                                        Parenthesized path arguments are not supported.");
            }
        }
        path.push('.');
    }
    path.remove(path.len() - 1); // remove last '.'
    if generics.len() > 0 {
        TypeRef::Generic(Box::new(TypeRef::Name(path)), generics)
    }
    else {
        TypeRef::Name(path)
    }
}

fn get_visibility(vis: &Visibility) -> TgVis {
    match vis {
        Visibility::Public(_) => TgVis::Public,
        Visibility::Restricted(sub_vis) => {
            if let TypeRef::Name(sub_vis_name) = get_from_path(&sub_vis.path) {
                if sub_vis_name == "super" {
                    TgVis::Protected
                } else {
                    TgVis::Internal
                }
            }
            else {
                println!("[Warning] (tangara-gen::PackageGenerator) Strange visibility path. Set to Private");
                TgVis::Private
            }
        }
        Visibility::Inherited => TgVis::Private
    }
}

fn get_attr_lifetime(lifetime: &Lifetime) -> Attribute {
    RUST_STD_LIB.lifetime_attribute(&lifetime.ident.to_string())
}

fn get_value(expr: &Expr) -> Option<Value> {
    match expr {
        Expr::Array(array_expr) => {
            let array_values = array_expr.elems.iter()
                .map(|item_expr| get_value(item_expr).expect("Value of array can't be None"))
                .collect();
            Some(Value::Array(array_values))
        }
        Expr::Lit(lit_expr) => {
            match &lit_expr.lit {
                Lit::Str(str_lit) => Some(Value::String(str_lit.value())),
                Lit::ByteStr(bstr_lit) => {
                    Some(Value::Array(
                        bstr_lit.value().iter().map(|&byte| Value::Byte(byte)).collect()
                    ))
                },
                Lit::Byte(byte_lit) => Some(Value::Byte(byte_lit.value())),
                Lit::Char(char_lit) => Some(Value::UInt(char_lit.value() as u32)),
                Lit::Int(int_lit) => {
                    Some(match int_lit.suffix() {
                        "i8" => Value::SByte(int_lit.base10_parse().expect("Expected i8 value")),
                        "u8" => Value::Byte(int_lit.base10_parse().expect("Expected u8 value")),
                        "i16" => Value::Short(int_lit.base10_parse().expect("Expected i16 value")),
                        "u16" => Value::UShort(int_lit.base10_parse().expect("Expected u16 value")),
                        "u32" => Value::UInt(int_lit.base10_parse().expect("Expected u32 value")),
                        "i64" => Value::Long(int_lit.base10_parse().expect("Expected i64 value")),
                        "u64" => Value::ULong(int_lit.base10_parse().expect("Expected u64 value")),
                        _ => Value::Int(int_lit.base10_parse().expect("Expected i32 value"))
                    })
                },
                Lit::Float(float_lit) => {
                    Some(match float_lit.suffix() {
                        "f32" => Value::Float(float_lit.base10_parse().expect("Expected f32 value")),
                        _ => Value::Double(float_lit.base10_parse().expect("Expected f64 value")),
                    })
                },
                Lit::Bool(bool_lit) => Some(Value::Bool(bool_lit.value())),
                _ => None
            }
        }
        Expr::Paren(paren_expr) => {
            get_value(&paren_expr.expr)
        }
        Expr::Struct(expr_struct) => {
            let mut object = HashMap::with_capacity(expr_struct.fields.len());
            for field in &expr_struct.fields {
                match &field.member {
                    Member::Named(named_field) => {
                        object.insert(
                            named_field.to_string(),
                            Box::new(get_value(&field.expr).expect("Value of object's field can't be None"))
                        );
                    }
                    Member::Unnamed(_) => {
                        println!("[Warning] (tangara-gen::PackageGenerator) \
                    Unnamed fields in struct expr doesn't supported");
                        return None;
                    }
                }
            }
            Some(Value::Object(object))
        }
        _ => None
    }
}

fn get_typeref(t: &Type) -> Option<(TypeRef, Vec<Attribute>)> {
    match t {
        Type::Array(array_type) => {
            let arr_len = get_value(&array_type.len).expect("Array length value can't be None");
            let mut attrs = vec![Attribute(
                TypeRef::from("Tangara.Metadata.ArraySize"), vec![arr_len]
            )];
            let (array_type, mut arr_attrs) = get_typeref(&array_type.elem).expect("Array type can't be None");
            attrs.append(&mut arr_attrs);
            Some((
                TypeRef::Generic(
                    Box::new(TypeRef::from("Tangara.Std.Array")),
                    vec![array_type]
                ),
                attrs
            ))
        },
        Type::BareFn(fn_type) => {
            // Parse return type
            let return_type = match &fn_type.output {
                ReturnType::Default => None,
                ReturnType::Type(_, ret_type) => {
                    if let Some((ret_typeref, _)) = get_typeref(ret_type) {
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
                let (arg_type, _) = get_typeref(&input.ty).expect("Argument type cannot be None");
                args.push(arg_type);
            }

            Some((TypeRef::Fn(return_type, args), vec![]))
        },
        Type::Group(_) => None,
        Type::ImplTrait(_) => None,
        Type::Macro(_) => None,
        Type::Never(_) => None,
        Type::Paren(paren_type) => {
            get_typeref(&paren_type.elem)
        },
        Type::Path(path_type) => {
            // we can't use `get_path_type` there because else we can get duplicated generics
            let typeref = TypeRef::from(
                path_type.path.segments.iter().map(|seg| {
                    seg.ident.to_string()
                }).collect::<Vec<String>>().join(".")
            );
            if let Some(last_seg) = &path_type.path.segments.last() {
                match &last_seg.arguments {
                    PathArguments::None => {
                        Some((typeref, vec![]))
                    }
                    PathArguments::AngleBracketed(angle_path) => {
                        let mut generics = vec![];
                        let mut attrs = vec![];
                        for generic in &angle_path.args {
                            match &generic {
                                GenericArgument::Lifetime(lifetime) => {
                                    attrs.push(get_attr_lifetime(lifetime));
                                }
                                GenericArgument::Type(generic_type) => {
                                    let (gtref, _) = get_typeref(generic_type).expect("Generic type can't be None");
                                    generics.push(gtref);
                                }
                                _ => {
                                    println!("[Warning] (tangara-gen::PackageGenerator) Other generics \
                                    types are not supported");
                                }
                            }
                        }
                        if generics.len() > 0 {
                            Some((TypeRef::Generic(Box::new(typeref), generics), attrs))
                        } else {
                            Some((typeref, attrs))
                        }
                    }
                    PathArguments::Parenthesized(_) => {
                        println!("[Warning] (tangara-gen::PackageGenerator) What parenthesized path type mean?");
                        Some((typeref, vec![]))
                    }
                }
            } else {
                Some((typeref, vec![]))
            }
        },
        Type::Ptr(ptr_type) => {
            let mut attrs = vec![];
            if ptr_type.mutability.is_some() {
                attrs.push(RUST_STD_LIB.mutable_attribute())
            }
            let (ptr_typeref, mut ptr_attrs) = get_typeref(&ptr_type.elem).expect("Pointer type cannot be None");
            attrs.append(&mut ptr_attrs);
            Some((
                TypeRef::Generic(
                    Box::new(TypeRef::from("Tangara.Std.Ptr")),
                    vec![ptr_typeref]
                ),
                attrs
            ))
        },
        Type::Reference(ref_type) => {
            let mut attrs = vec![RUST_STD_LIB.reference_attribute()];
            if let Some(lifetime) = &ref_type.lifetime {
                attrs.push(get_attr_lifetime(lifetime));
            }
            if ref_type.mutability.is_some() {
                attrs.push(RUST_STD_LIB.mutable_attribute())
            }
            let (ref_type, mut ref_attrs) = get_typeref(&ref_type.elem).expect("Reference type can't be None");
            attrs.append(&mut ref_attrs);
            Some((ref_type, attrs))
        },
        Type::Slice(slice_type) => {
            let (slice_typeref, attrs) = get_typeref(&slice_type.elem).expect("Slice type cannot be None");
            Some((
                TypeRef::Generic(
                    Box::new(TypeRef::from("Tangara.Std.Array")),
                    vec![slice_typeref]
                ),
                attrs
            ))
        },
        Type::TraitObject(_) => None,
        Type::Tuple(tuple_type) => {
            let mut types = vec![];
            for tt in &tuple_type.elems {
                let ott = get_typeref(tt);
                if ott.is_some() {
                    types.push(ott.unwrap().0);
                }
            }
            Some((TypeRef::Tuple(types), vec![]))
        }
        _ => None
    }
}

fn parse_return_type<T: MethodCollector>(fn_builder: &mut MethodBuilder<T>, return_type: &ReturnType) {
    match return_type {
        ReturnType::Default => {} // return type of fn_builder by default is nothing
        ReturnType::Type(_, ret_type) => {
            if let Some((ret_typeref, ret_attrs)) = get_typeref(ret_type) {
                if RUST_STD_LIB.is_reference(&ret_attrs) {
                    let mut return_prefix = "&".to_string();
                    if let Some(lifetime) = RUST_STD_LIB.get_lifetime(&ret_attrs) {
                        return_prefix = format!("&'{lifetime} ");
                    }
                    if RUST_STD_LIB.is_mutable(&ret_attrs) {
                        return_prefix = format!("{return_prefix}mut ");
                    }
                    fn_builder.add_attribute(RUST_STD_LIB.return_attribute(&return_prefix));
                }
                fn_builder.return_type(ret_typeref);
            }
        }
    }
}

fn parse_arg<T: MethodCollector>(fn_builder: &mut MethodBuilder<T>, fn_arg: &PatType) {
    if let Pat::Ident(arg_ident) = &fn_arg.pat.deref() {
        let arg_name = arg_ident.ident.to_string();
        let arg_type = get_typeref(&fn_arg.ty).expect("Arg type cannot be None");
        for attr in &arg_type.1 {
            fn_builder.arg_attribute(attr.clone());
        }
        if arg_ident.mutability.is_some() {
            fn_builder.arg_ref(arg_type.0, arg_name.as_str());
        }
        else if RUST_STD_LIB.is_reference(&arg_type.1) {
            if RUST_STD_LIB.is_mutable(&arg_type.1) {
                fn_builder.arg_ref(arg_type.0, arg_name.as_str());
            }
            else {
                fn_builder.arg_in(arg_type.0, arg_name.as_str());
            }
        }
        else {
            fn_builder.arg(arg_type.0, arg_name.as_str());
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
                    let typeref_wheres = get_from_path(&trait_bound.path);
                    generic_wheres.push((bounded.clone(), typeref_wheres));
                }
                TypeParamBound::Lifetime(lifetime) => {
                    // Add attribute to mark for Tangara that in Rust this generic has lifetime
                    let lt = lifetime.ident.to_string();
                    builder.add_attribute(
                        RUST_STD_LIB.lifetime_generic_attribute(&bounded, &lt)
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
                builder.add_attribute(get_attr_lifetime(&lifetime.lifetime));
            }
            GenericParam::Const(_) => {
                println!("[Warning] (tangara-gen::PackageGenerator) Const are not supported in generics.");
            }
        }
    }

    if let Some(where_clause) = &generics.where_clause {
        for predicate in &where_clause.predicates {
            match predicate {
                WherePredicate::Lifetime(_) => {
                    println!("[Warning] (tangara-gen::PackageGenerator) Lifetimes are not supported \
                    in 'where' clauses.");
                }
                WherePredicate::Type(type_predicate) => {
                    if let Some((TypeRef::Name(type_name), _)) = get_typeref(&type_predicate.bounded_ty) {
                        parse_bounds(builder, type_name, &type_predicate.bounds);
                    }
                    else {
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
            package_builder: PackageBuilder::new(package_name, NamingConventions::rust()),
            structs: HashMap::new()
        }
    }

    fn get_or_create_struct(&mut self, name: String) -> &mut ClassBuilder {
        self.structs.entry(name.clone()).or_insert(create_class(self.package_builder.clone(), &name))
    }

    fn parse_item(&mut self, item: &Item) {
        match item {
            Item::Enum(enum_item) => {
                let enum_name = enum_item.ident.to_string();
                let enum_vis = get_visibility(&enum_item.vis);
                let is_enum_class = enum_item.variants.iter().any(|v| v.fields != Fields::Unit);
                if is_enum_class {
                    let mut builder = create_enum_class(self.package_builder.clone(), &enum_name);
                    builder.set_visibility(enum_vis);
                    parse_generics(&mut builder, &enum_item.generics);
                    for variant in &enum_item.variants {
                        let mut variant_builder = builder.variant(&variant.ident.to_string());
                        // Count of fields
                        let mut count = 0;
                        if let Fields::Unnamed(_) = &variant.fields {
                            variant_builder.add_attribute(RUST_STD_LIB.tuple_variant_attribute());
                        }
                        for field in &variant.fields {
                            let field_name = if let Some(field_ident) = &field.ident {
                                field_ident.to_string()
                            }
                            else {
                                format!("field{}", count)
                            };
                            let (field_type, field_attrs) = get_typeref(&field.ty)
                                .expect("Field cannot have type None");
                            let mut field_builder = variant_builder.add_field(field_type, &field_name);
                            if field.ident.is_none() {
                                field_builder.add_attribute(RUST_STD_LIB.tuple_field_attribute(count));
                            }
                            for attr in field_attrs {
                                field_builder.add_attribute(attr);
                            }
                            field_builder.add_attribute(RUST_STD_LIB.struct_field_attribute());
                            field_builder.set_visibility(get_visibility(&field.vis));
                            field_builder.build();
                            count += 1;
                        }
                        variant_builder.build();
                    }
                    builder.build();
                }
                else {
                    let mut builder = create_enum(self.package_builder.clone(), &enum_name);
                    builder.set_visibility(enum_vis);
                    for variant in &enum_item.variants {
                        let variant_name = variant.ident.to_string();
                        if let Some((_, lit_value)) = &variant.discriminant {
                            builder.variant_value(
                                &variant_name,
                                get_value(lit_value).expect("Value of enum cannot be None")
                            );
                        }
                        else {
                            builder.variant(&variant_name);
                        }
                    }
                    builder.build();
                }
            }
            Item::Impl(impl_item) => {
                let mut for_type = None;
                // Check situation on 'impl Trait for Struct'
                if let Some((_, type_name, _)) = &impl_item.trait_ {
                    for_type = Some(get_from_path(type_name));
                }
                if let (TypeRef::Name(type_name), _) = get_typeref(&impl_item.self_ty)
                    .expect("Type in 'impl' cannot be None") {
                    let ctor_names = self.config.ctor_names.to_vec();
                    let dont_inherit_traits = self.config.dont_inherit_traits.to_vec();

                    let cb = self.get_or_create_struct(type_name.clone());
                    if let Some(trait_type) = for_type {
                        // Again, if impl is with trait, then we need to inherit class from it
                        // But if really needs to. Because some traits is not important to inherit from.
                        match &trait_type {
                            TypeRef::Name(trait_name) => {
                                if !dont_inherit_traits.contains(&trait_name) {
                                    cb.inherits(trait_type);
                                }
                            }
                            TypeRef::Generic(trait_type_owner, _) => {
                                if let TypeRef::Name(trait_name) = trait_type_owner.deref() {
                                    if !dont_inherit_traits.contains(&trait_name) {
                                        cb.inherits(trait_type);
                                    }
                                }
                                else {
                                    println!("[Warning] (tangara-gen::PackageGenerator) TypeRef from \
                                generic not supported in 'impl TRef<...> for T' statement. Inherit from \
                                anyway.");
                                    cb.inherits(trait_type);
                                }
                            }
                            _ => {
                                println!("[Warning] (tangara-gen::PackageGenerator) TypeRef not \
                                supported in 'impl TRef for T' statement. Inherit from it anyway.");
                                cb.inherits(trait_type);
                            }
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
                                    ctor_builder.add_attribute(RUST_STD_LIB.constructor_name_attribute(&name));

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
                                            if let TypeRef::Name(return_type) = get_typeref(return_type_boxed).expect("").0 {
                                                if return_type != "Self" &&
                                                    return_type != type_name {
                                                    panic!("Return type of constructor can't be not as type of impl: {} != {}",
                                                           return_type, type_name);
                                                }
                                            }
                                            else {
                                                panic!("Return type reference of constructor is not Name");
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
                                                    for attr in &arg_type.1 {
                                                        ctor_builder.arg_attribute(attr.clone());
                                                    }
                                                    if arg_ident.mutability.is_some() {
                                                        ctor_builder.arg_ref(arg_type.0, arg_name.as_str());
                                                    }
                                                    else if RUST_STD_LIB.is_reference(&arg_type.1) {
                                                        if RUST_STD_LIB.is_mutable(&arg_type.1) {
                                                            ctor_builder.arg_ref(arg_type.0, arg_name.as_str());
                                                        }
                                                        else {
                                                            ctor_builder.arg_in(arg_type.0, arg_name.as_str());
                                                        }
                                                    }
                                                    else {
                                                        ctor_builder.arg(arg_type.0, arg_name.as_str());
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
                                    let mut is_self = false;
                                    let mut is_self_mut = false;
                                    let mut is_self_ref = false;
                                    for arg in &fn_sig.inputs {
                                        match arg {
                                            FnArg::Receiver(self_arg) => {
                                                // TODO add handling of lifetime
                                                is_self = true;
                                                is_self_mut = self_arg.mutability.is_some();
                                                is_self_ref = self_arg.reference.is_some();
                                            }
                                            FnArg::Typed(fn_arg) => {
                                                // TODO add checks on Self type
                                                parse_arg(&mut fn_builder, fn_arg);
                                            }
                                        }
                                    }
                                    if is_self {
                                        if is_self_mut {
                                            fn_builder.add_attribute(RUST_STD_LIB.mutable_attribute());
                                        }
                                        if is_self_ref {
                                            fn_builder.add_attribute(RUST_STD_LIB.reference_attribute());
                                        }
                                    } else {
                                        fn_builder.set_kind(MethodKind::Static);
                                    }

                                    fn_builder.build();
                                }
                            }
                            ImplItem::Type(_) => {} // TODO add checks in typeref making in function (return or args) on this type
                            _ => {}
                        }
                    }
                } // if let TypeRef::Name(type_name) = get_typeref(&impl_item.self_ty)
                else {
                    println!("[Warning] (tangara-gen::PackageGenerator) TypeRef from 'impl' root must be Name");
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
                let class_builder = self.get_or_create_struct(struct_item.ident.to_string());
                class_builder.set_visibility(get_visibility(&struct_item.vis));
                parse_generics(class_builder, &struct_item.generics);

                let mut count = 0;
                for field in &struct_item.fields {
                    let field_name = if let Some(field_ident) = &field.ident {
                        field_ident.to_string()
                    }
                    else {
                        format!("field{}", count)
                    };
                    let (field_type, field_attrs) = get_typeref(&field.ty)
                        .expect("Field cannot have type None");
                    let mut field_builder = class_builder.add_field(field_type, &field_name);
                    if field.ident.is_none() {
                        field_builder.add_attribute(RUST_STD_LIB.tuple_field_attribute(count));
                    }
                    for attr in field_attrs {
                        field_builder.add_attribute(attr);
                    }
                    field_builder.add_attribute(RUST_STD_LIB.struct_field_attribute());
                    field_builder.set_visibility(get_visibility(&field.vis));
                    field_builder.build();
                    count += 1;
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
                            let mut is_self = false;
                            let mut is_self_mut = false;
                            let mut is_self_ref = false;
                            for arg in &fn_item.sig.inputs {
                                match arg {
                                    FnArg::Receiver(self_arg) => {
                                        // TODO add handling of lifetime
                                        is_self = true;
                                        is_self_mut = self_arg.mutability.is_some();
                                        is_self_ref = self_arg.reference.is_some();
                                    }
                                    FnArg::Typed(fn_arg) => {
                                        parse_arg(&mut fn_builder, fn_arg);
                                    }
                                }
                            }
                            if is_self {
                                if is_self_mut {
                                    fn_builder.add_attribute(RUST_STD_LIB.mutable_attribute());
                                }
                                if is_self_ref {
                                    fn_builder.add_attribute(RUST_STD_LIB.reference_attribute());
                                }
                                fn_builder.build();
                            }
                            else {
                                println!("[Warning] (tangara-gen::PackageGenerator) Trait \
                                 (interface) method must have 'self' argument. Ignoring it.");
                            }
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
                    get_typeref(&type_item.ty).expect("Type in alias cannot be None").0
                );
                alias_builder.set_visibility(get_visibility(&type_item.vis));
                parse_generics(&mut alias_builder, &type_item.generics);
                alias_builder.build();
            }
            _ => {}
        }
    }

    /// Set full path of mod as namespace. Mod path must be `my_mod::extra` format.
    /// If mod path was set earlier - it rewrites it.
    pub fn set_mod(mut self, mod_path: &str) -> Self {
        self.package_builder.borrow_mut().set_namespace(mod_path);
        self
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
            if let TypeKind::Class {
                is_sealed: _is_sealed,
                constructors,
                properties,
                fields,
                static_properties,
                static_fields,
                methods,
                parents
            } = &result.kind {
                let mut builder = self.package_builder.borrow_mut();
                builder.add_type(
                    // Change type's kind on Struct if it's possible
                    if methods.len() == 0 &&
                        parents.len() == 0 &&
                        properties.len() == 0 &&
                        static_properties.len() == 0 {
                        let mut result = result.clone();
                        result.kind = TypeKind::Struct {
                            constructors: constructors.to_vec(),
                            fields: fields.to_vec(),
                            static_fields: static_fields.to_vec()
                        };
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