use tangara_gen::Generator;

#[test]
fn gentest() {
    /*std::fs::write(
        "tests/result.rs",
        Generator::new("MyPackage")
            .generate_internal(true)
            .parse_file("tests/mymod.rs")
            .custom_uses(vec!["crate::mymod"])
            .generate()
            .to_string()
    ).unwrap();*/

    Generator::new("Tangara")
        .parse_file("../src/lib.rs")
        .parse_file("../src/context.rs")
        .parse_file("../src/runtime.rs")
        .custom_uses(vec![
            "tangara::context::*",
            "tangara::runtime::*",
        ])
        .generate_to_file("tests/tangara.rs")
        .unwrap();
}