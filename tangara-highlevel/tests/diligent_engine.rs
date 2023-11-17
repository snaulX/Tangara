use std::collections::HashMap;
use tangara_highlevel::builder::*;

enum Number {
    Int(i64),
    UInt(u64),
    Float(f64)
}

enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

#[test]
fn diligent_engine() {
    /*let diligentengine_package = Package {
        name: "".to_string(),
        id: 0,
    };*/
    let alphaengine = PackageBuilder::new("AlphaEngine");
    let isystem = create_interface(alphaengine.clone(), "ISystem");
}