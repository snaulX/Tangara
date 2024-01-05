use once_cell::sync::Lazy;
use tangara_highlevel::{Attribute, get_typeref_bytes, Package, Type, TypeRef, Value, Visibility};
use tangara_highlevel::builder::{create_class, PackageBuilder, TypeBuilder};

mod package_generator;
mod rust_generator;
mod entrypoint_generator;
mod source_generator;

pub use package_generator::PackageGenerator;
pub use package_generator::Config as PkgGenConfig;
pub use rust_generator::RustGenerator;
pub use rust_generator::Config as RustGenConfig;

pub static RUST_STD_LIB: Lazy<RustStdLib> = Lazy::new(RustStdLib::new);

pub struct RustStdLib {
    rust_std: Package,
    mutable_attribute: Type,
    struct_field_attribute: Type,
    reference_attribute: Type,
    lifetime_attribute: Type,
    lifetime_generic_attribute: Type,
    constructor_name_attribute: Type,
    tuple_field_attribute: Type,
    tuple_variant_attribute: Type,
}

impl RustStdLib {
    pub(crate) fn new() -> Self {
        let mut rust_std = PackageBuilder::new("Tangara.Rust");
        {
            let mut rust_std_ref = rust_std.borrow_mut();
            rust_std_ref.type_visibility = Visibility::Public;
            rust_std_ref.member_visibility = Visibility::Public;
            rust_std_ref.method_visibility = Visibility::Public;
            rust_std_ref.constructor_visibility = Visibility::Public;
            rust_std_ref.set_namespace("Tangara.Rust.Metadata");
        }
        // TODO inherits from Tangara.Std.Attribute
        let mut struct_field_attribute = create_class(rust_std.clone(), "StructField");
        let mut mutable_attribute = create_class(rust_std.clone(), "Mutable");
        let mut reference_attribute = create_class(rust_std.clone(), "Reference");
        let mut lifetime_attribute = create_class(rust_std.clone(), "Lifetime");
        lifetime_attribute.add_property(TypeRef::from("String"), "Name")
            .setter_visibility(Visibility::Public).build();
        let mut lifetime_generic_attribute = create_class(rust_std.clone(), "LifetimeGeneric");
        lifetime_generic_attribute.add_property(TypeRef::from("String"), "Name")
            .setter_visibility(Visibility::Public).build();
        let mut constructor_name_attribute = create_class(rust_std.clone(), "ConstructorFnName");
        constructor_name_attribute.add_property(TypeRef::from("String"), "FnName")
            .setter_visibility(Visibility::Public).build();
        let mut tuple_field_attribute = create_class(rust_std.clone(), "TupleField");
        tuple_field_attribute.add_property(TypeRef::from("UShort"), "Index")
            .setter_visibility(Visibility::Public).build();
        let mut tuple_variant_attribute = create_class(rust_std.clone(), "TupleVariant");

        // Build classes
        let struct_field_attribute = struct_field_attribute.build();
        let mutable_attribute = mutable_attribute.build();
        let reference_attribute = reference_attribute.build();
        let lifetime_attribute = lifetime_attribute.build();
        let lifetime_generic_attribute = lifetime_generic_attribute.build();
        let constructor_name_attribute = constructor_name_attribute.build();
        let tuple_field_attribute = tuple_field_attribute.build();
        let tuple_variant_attribute = tuple_variant_attribute.build();
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
            tuple_variant_attribute,
        }
    }

    pub fn get_package(&self) -> Package {
        self.rust_std.clone()
    }

    pub fn mutable_attribute(&self) -> Attribute {
        Attribute(TypeRef::from(&self.mutable_attribute), vec![])
    }

    pub fn struct_field_attribute(&self) -> Attribute {
        Attribute(TypeRef::from(&self.struct_field_attribute), vec![])
    }

    pub fn reference_attribute(&self) -> Attribute {
        Attribute(TypeRef::from(&self.reference_attribute), vec![])
    }

    pub fn lifetime_attribute(&self, lifetime: &str) -> Attribute {
        Attribute(TypeRef::from(&self.lifetime_attribute), vec![Value::from(lifetime)])
    }

    pub fn lifetime_generic_attribute(&self, bounded: &str, lifetime: &str) -> Attribute {
        Attribute(TypeRef::from(&self.lifetime_generic_attribute),
                  vec![Value::from(bounded), Value::from(lifetime)])
    }

    pub fn constructor_name_attribute(&self, fn_name: &str) -> Attribute {
        Attribute(TypeRef::from(&self.constructor_name_attribute), vec![Value::from(fn_name)])
    }

    pub fn tuple_field_attribute(&self, index: u16) -> Attribute {
        Attribute(TypeRef::from(&self.tuple_field_attribute), vec![Value::from(index)])
    }

    pub fn tuple_variant_attribute(&self) -> Attribute {
        Attribute(TypeRef::from(&self.tuple_variant_attribute), vec![])
    }

    pub fn is_struct_field(&self, attrs: &[Attribute]) -> bool {
        // Cache type data for comparing
        let struct_field_data = get_typeref_bytes(&TypeRef::from(&self.struct_field_attribute));
        attrs.iter().any(|attr| get_typeref_bytes(&attr.0) == struct_field_data)
    }

    pub fn is_mutable(&self, attrs: &[Attribute]) -> bool {
        // Cache type data for comparing
        let mutable_data = get_typeref_bytes(&TypeRef::from(&self.mutable_attribute));
        attrs.iter().any(|attr| get_typeref_bytes(&attr.0) == mutable_data)
    }

    pub fn is_reference(&self, attrs: &[Attribute]) -> bool {
        // Cache type data for comparing
        let reference_data = get_typeref_bytes(&TypeRef::from(&self.reference_attribute));
        attrs.iter().any(|attr| get_typeref_bytes(&attr.0) == reference_data)
    }

    /// Check if variant of enum class is tuple
    pub fn is_tuple_variant(&self, attrs: &[Attribute]) -> bool {
        // Cache type data for comparing
        let tuple_variant_data = get_typeref_bytes(&TypeRef::from(&self.tuple_variant_attribute));
        attrs.iter().any(|attr| get_typeref_bytes(&attr.0) == tuple_variant_data)
    }

    /// Check attributes on `ConstructorFnName` attribute and returns his 1st value (`FnName`) if it exists.
    pub fn get_fn_name(&self, attrs: &[Attribute]) -> Option<String> {
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

    /// Check attributes on `TupleField` attribute and returns his 1st value (`Index`) if it exists.
    pub fn get_tuple_index(&self, attrs: &[Attribute]) -> Option<u16> {
        let tuple_field_data = get_typeref_bytes(&TypeRef::from(&self.tuple_field_attribute));
        attrs.iter().find_map(|attr| {
            if get_typeref_bytes(&attr.0) == tuple_field_data {
                if let Value::UShort(index) = &attr.1[0] {
                    return Some(index.clone());
                }
            }
            None
        })
    }
}