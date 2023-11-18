use tangara_gen::*;

fn main() {
    /*Generator::new("MyPackage")
        .enable_internal(true)
        .parse_file("src/lib.rs")
        .custom_uses(vec!["crate::*"])
        .generate_to_file("src/bindings.rs")
        .unwrap();*/

    let pkg = PackageGenerator::new("MyPackage", Config::default())
        .parse_file("src/lib.rs")
        .generate();
    let pkg_json = serde_json::to_string_pretty(&pkg).expect("Convert tangara package to json");
    std::fs::write("refldata.json", pkg_json).expect("Error with writing to refldata.json");

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