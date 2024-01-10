use libloading::Symbol;
use tangara::context::Context;
use tangara::runtime::Runtime;
use crate::bindings::{load_MyLib, MyStruct, TestStruct};

mod bindings;

fn main() {
    unsafe {
        // load dynamic library and tangara functions from them
        let lib = libloading::Library::new("mylib").expect("Loading mylib dynamic library gets errors");
        let tgload: Symbol<unsafe extern fn(&mut Context)> = lib.get(b"tgLoad").unwrap();
        {
            // bindings can works further even without keeping tangara context in memory
            let mut runtime = Runtime::new(); // runtime need to be more enhanced to support custom allocators and etc.
            let mut context = runtime.use_context();
            tgload(&mut context); // load lib from dll to context
            load_MyLib(&context); // load bindings from context
        }
        // use bindings
        // Note: don't move this code out of unsafe because lib gets unloaded and
        // every function pointer in bindings become invalid
        let mut test = TestStruct::new(64);
        println!("Id #1: {}", test.get_id());
        test.set_id(164);
        println!("Id #2: {}", test.get_id());
        let mut snaulx = MyStruct::new("snaulX");
        println!("{} is author of this library", snaulx.get_name());
        snaulx.set_name("https://github.com/snaulX");
        snaulx.repeat_name(5);
    }
}
