mod bindings;

trait WarningTrait {
    fn kekov() -> i32;
}

enum EnumUnit {
    Variant
}

pub(crate) enum EnumTuple {
    Variant(i32)
}

pub enum EnumStruct {
    Variant {
        a: i32
    }
}

enum EnumMixed {
    Unit,
    Tuple(i32)
}

enum EnumComplex {
    Unit,
    Tuple(i32),
    Struct {
        a: i32
    }
}

pub trait MyTrait {
    fn foo(&mut self, a: String);
    fn bar(&self) -> String;
}

pub struct GenericsTest<T: MyTrait> {
    some_field: Option<T>
}

pub struct TestStruct {
    pub id: u64
}

pub struct MyStruct {
    name: String
}

pub(crate) type BoxedStr = Box<str>;

impl MyTrait for MyStruct {
    fn foo(&mut self, a: String) {
        self.name = a;
    }

    fn bar(&self) -> String {
        self.name.clone()
    }
}

impl MyStruct {
    pub(crate) fn new() -> Self {
        Self {
            name: "snaulX".to_string()
        }
    }

    pub(crate) fn repeat_name(&self, times: u32) -> () {
        for _ in 0..times {
            println!("My name is {}", self.name);
        }
    }

    pub(crate) fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }
}