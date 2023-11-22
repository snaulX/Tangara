use std::cell::RefCell;
use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;
use once_cell::sync::Lazy;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{FnArg, ImplItem, Item, parse_file, Pat, PatType, ReturnType, Signature, Type, UseTree, Visibility};
use xxhash_rust::const_xxh3::const_custom_default_secret;
use xxhash_rust::xxh3::xxh3_64_with_secret;
use tangara_highlevel::builder::{create_class, PackageBuilder, TypeBuilder};
use tangara_highlevel::{Attribute, get_typeref_bytes, Package, TypeRef, Value};
use tangara_highlevel::Type as TgType;
use tangara_highlevel::Visibility as TgVis;

mod package_generator;
mod rust_generator;

pub use package_generator::PackageGenerator;
pub use package_generator::Config as PkgGenConfig;
pub use rust_generator::RustGenerator;
pub use rust_generator::Config as RustGenConfig;

pub(crate) static RUST_STD_LIB: Lazy<RustStdLib> = Lazy::new(|| RustStdLib::new());

pub(crate) struct RustStdLib {
    rust_std: Package,
    mutable_attribute: TgType,
    struct_field_attribute: TgType,
    reference_attribute: TgType,
    lifetime_attribute: TgType,
    lifetime_generic_attribute: TgType,
    constructor_name_attribute: TgType,
    tuple_field_attribute: TgType,
}

impl RustStdLib {
    pub(crate) fn new() -> Self {
        let mut rust_std = PackageBuilder::new("Tangara.Rust");
        {
            let mut rust_std_ref = rust_std.borrow_mut();
            rust_std_ref.type_visibility = TgVis::Public;
            rust_std_ref.property_visibility = TgVis::Public;
            rust_std_ref.method_visibility = TgVis::Public;
            rust_std_ref.constructor_visibility = TgVis::Public;
            rust_std_ref.set_namespace("Tangara.Rust.Metadata");
        }
        // TODO inherits from Attribute
        let mut struct_field_attribute = create_class(rust_std.clone(), "StructField");
        let mut mutable_attribute = create_class(rust_std.clone(), "Mutable");
        let mut reference_attribute = create_class(rust_std.clone(), "Reference");
        let mut lifetime_attribute = create_class(rust_std.clone(), "Lifetime");
        lifetime_attribute.add_property(TypeRef::from("String"), "Name")
            .setter_visibility(TgVis::Public).build();
        let mut lifetime_generic_attribute = create_class(rust_std.clone(), "LifetimeGeneric");
        lifetime_generic_attribute.add_property(TypeRef::from("String"), "Name")
            .setter_visibility(TgVis::Public).build();
        let mut constructor_name_attribute = create_class(rust_std.clone(), "ConstructorFnName");
        constructor_name_attribute.add_property(TypeRef::from("String"), "FnName")
            .setter_visibility(TgVis::Public).build();
        let mut tuple_field_attribute = create_class(rust_std.clone(), "TupleField");
        tuple_field_attribute.add_property(TypeRef::from("UShort"), "Index")
            .setter_visibility(TgVis::Public).build();

        // Build classes
        let struct_field_attribute = struct_field_attribute.build();
        let mutable_attribute = mutable_attribute.build();
        let reference_attribute = reference_attribute.build();
        let lifetime_attribute = lifetime_attribute.build();
        let lifetime_generic_attribute = lifetime_generic_attribute.build();
        let constructor_name_attribute = constructor_name_attribute.build();
        let tuple_field_attribute = tuple_field_attribute.build();
        let rust_std = rust_std.borrow().build();

        Self {
            rust_std,
            mutable_attribute,
            struct_field_attribute,
            reference_attribute,
            lifetime_attribute,
            lifetime_generic_attribute,
            constructor_name_attribute,
            tuple_field_attribute,
        }
    }

    pub(crate) fn mutable_attribute(&self) -> Attribute {
        Attribute(TypeRef::from(&self.mutable_attribute), vec![])
    }

    pub(crate) fn struct_field_attribute(&self) -> Attribute {
        Attribute(TypeRef::from(&self.struct_field_attribute), vec![])
    }

    pub(crate) fn reference_attribute(&self) -> Attribute {
        Attribute(TypeRef::from(&self.reference_attribute), vec![])
    }

    pub(crate) fn lifetime_attribute(&self, lifetime: &str) -> Attribute {
        Attribute(TypeRef::from(&self.lifetime_attribute), vec![Value::from(lifetime)])
    }

    pub(crate) fn lifetime_generic_attribute(&self, bounded: &str, lifetime: &str) -> Attribute {
        Attribute(TypeRef::from(&self.lifetime_generic_attribute),
                  vec![Value::from(bounded), Value::from(lifetime)])
    }

    pub(crate) fn constructor_name_attribute(&self, fn_name: &str) -> Attribute {
        Attribute(TypeRef::from(&self.constructor_name_attribute), vec![Value::from(fn_name)])
    }

    pub(crate) fn tuple_field_attribute(&self, index: u16) -> Attribute {
        Attribute(TypeRef::from(&self.tuple_field_attribute), vec![Value::from(index)])
    }

    pub(crate) fn is_struct_field(&self, attrs: &[Attribute]) -> bool {
        // Cache type data for comparing
        let struct_field_data = get_typeref_bytes(&TypeRef::from(&self.struct_field_attribute));
        attrs.iter().any(|attr| get_typeref_bytes(&attr.0) == struct_field_data)
    }

    pub(crate) fn is_mutable(&self, attrs: &[Attribute]) -> bool {
        // Cache type data for comparing
        let mutable_data = get_typeref_bytes(&TypeRef::from(&self.mutable_attribute));
        attrs.iter().any(|attr| get_typeref_bytes(&attr.0) == mutable_data)
    }

    pub(crate) fn is_reference(&self, attrs: &[Attribute]) -> bool {
        // Cache type data for comparing
        let reference_data = get_typeref_bytes(&TypeRef::from(&self.reference_attribute));
        attrs.iter().any(|attr| get_typeref_bytes(&attr.0) == reference_data)
    }

    /// Check attributes on `ConstructorFnName` attribute and returns his 1st value (`FnName`) if it exists.
    pub(crate) fn get_fn_name(&self, attrs: &[Attribute]) -> Option<String> {
        let constructor_name_data = get_typeref_bytes(&TypeRef::from(&self.constructor_name_attribute));
        attrs.iter().find_map(|attr| {
            if get_typeref_bytes(&attr.0) == constructor_name_data {
                if let Value::String(name) = &attr.1[0] {
                    return Some(name.clone());
                }
            }
            None
        })
    }
}