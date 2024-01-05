use tangara_gen::{RustGenConfig, RustGenerator};
use tangara_highlevel::Package;

fn main() {
    let p: Package = serde_json::from_str(&std::fs::read_to_string("../mylib.tgjson").unwrap()).unwrap();
    RustGenerator::new(p, RustGenConfig::default())
        .generate_bindings()
        .write_to("src/bindings.rs")
        .unwrap();
}