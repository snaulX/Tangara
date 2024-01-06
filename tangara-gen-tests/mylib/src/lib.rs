mod bindings;

trait WarningTrait {
    fn kekov() -> i32;
}

pub enum EnumUnit {
    Variant
}

pub enum EnumTuple {
    Variant(i32)
}

pub enum EnumStruct {
    Variant {
        a: i32
    }
}

pub enum EnumMixed {
    Unit,
    Tuple(i32)
}

pub enum EnumComplex {
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

pub type BoxedStr = Box<str>;

impl TestStruct {
    pub fn new(id: u64) -> Self {
        Self {
            id
        }
    }
}

impl<T: MyTrait> GenericsTest<T> {
    pub fn new() -> Self {
        Self {
            some_field: None
        }
    }

    pub fn set_t(&mut self, t: T) {
        self.some_field = Some(t);
    }

    pub fn print_t_bar(&self) {
        if let Some(t) = &self.some_field {
            println!("Some({})", t.bar());
        }
        else {
            println!("None");
        }
    }
}

impl MyTrait for MyStruct {
    fn foo(&mut self, a: String) {
        self.name = a;
    }

    fn bar(&self) -> String {
        self.name.clone()
    }
}

impl MyStruct {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string()
        }
    }

    pub fn repeat_name(&self, times: u32) -> () {
        for _ in 0..times {
            println!("My name is {}", self.name);
        }
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}