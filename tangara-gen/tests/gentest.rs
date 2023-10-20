use tangara_gen::Generator;

#[test]
fn gentest() {
    Generator::new("MyPackage")
        .enable_internal(true)
        .parse_file("tests/mymod.rs")
        .generate_to_file("tests/result.rs")
        .unwrap();

    Generator::new("Tangara")
        .parse_file("../tangara/src/lib.rs")
        .parse_file("../tangara/src/context.rs")
        .parse_file("../tangara/src/runtime.rs")
        .custom_uses(vec![
            "tangara::context::*",
            "tangara::runtime::*",
        ])
        .generate_to_file("tests/tangara.rs")
        .unwrap();
}