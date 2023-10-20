struct MyStruct {
    name: String
}

impl MyStruct {
    pub(crate) fn new() -> Self {
        Self {
            name: "snaulX".to_string()
        }
    }

    pub(crate) fn repeat_name(&self, times: u32) -> () {
        for i in 0..times {
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