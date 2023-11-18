mod bindings;

pub trait MyTrait {
    fn foo(&mut self, a: &MyStruct);
    fn bar(&self) -> MyStruct;
}

pub struct TestStruct {
    pub id: u64
}

pub struct MyStruct {
    name: String
}

pub(crate) type BoxedStr = Box<str>;

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