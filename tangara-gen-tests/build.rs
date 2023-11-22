use tangara_gen::*;
use tangara_highlevel::Package;

fn main() {
    let pkg = PackageGenerator::new("MyPackage", PkgGenConfig::default())
        .parse_file("src/lib.rs")
        .generate();
    let pkg_json = serde_json::to_string_pretty(&pkg).expect("Convert tangara package to json");
    std::fs::write("refldata.json", pkg_json).expect("Error with writing to refldata.json");

    let tangara = PackageGenerator::new("tangara", PkgGenConfig::default())
        .parse_file("../tangara/src/lib.rs")
        .set_mod("tangara::context")
        .parse_file("../tangara/src/context.rs")
        .set_mod("tangara::runtime")
        .parse_file("../tangara/src/runtime.rs")
        .generate();
    let tg_json = serde_json::to_string_pretty(&tangara).expect("Convert tangara package to json");
    std::fs::write("tangara.json", tg_json).expect("Error with writing to tangara.json");

    let p: Package = serde_json::from_str(&std::fs::read_to_string("refldata.json").unwrap()).unwrap();
    RustGenerator::new(p, RustGenConfig::default())
        .generate_entrypoint()
        .custom_use("crate::*")
        .write_to("src/bindings.rs")
        .unwrap();
}