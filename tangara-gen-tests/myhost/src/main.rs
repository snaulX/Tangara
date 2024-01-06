use libloading::Symbol;
use tangara::context::Context;
use tangara::runtime::Runtime;
use crate::bindings::{load_MyLib, TestStruct};

mod bindings;

fn main() {
    unsafe {
        // load dynamic library and tangara functions from them
        let lib = libloading::Library::new("mylib").unwrap();
        let tgload: Symbol<unsafe extern fn(&mut Context)> = lib.get(b"tgLoad").unwrap();
        let mut runtime = Runtime::new();
        let mut context = runtime.use_context();
        tgload(&mut context);
        load_MyLib(&context);
        // use bindings
        let mut test = TestStruct::new(64);
        println!("Id #1: {}", test.get_id());
        test.set_id(164);
        println!("Id #2: {}", test.get_id());
    }
}
