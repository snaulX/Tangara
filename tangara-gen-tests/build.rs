use tangara_gen::Generator;

fn main() {
    Generator::new("MyPackage")
        .enable_internal(true)
        .parse_file("src/lib.rs")
        .custom_uses(vec!["crate::*"])
        .generate_to_file("src/bindings.rs")
        .unwrap();

    /*Generator::new("Tangara")
        .parse_file("../tangara/src/lib.rs")
        .parse_file("../tangara/src/context.rs")
        .parse_file("../tangara/src/runtime.rs")
        .custom_uses(vec![
            "crate::context::*",
            "crate::runtime::*",
        ])
        .generate_to_file("../tangara/src/bindings.rs")
        .unwrap();*/
}