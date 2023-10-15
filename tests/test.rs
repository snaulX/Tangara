use std::alloc::{dealloc, Layout};
use std::ptr;
use tangara::context::Ptr;
use tangara::runtime::Runtime;

struct MyStruct {
    name: String
}

impl MyStruct {
    pub(crate) fn new() -> Self {
        Self {
            name: "snaulX".to_string()
        }
    }

    pub(crate) fn repeat_name(&self, times: u32) {
        for i in 0..times {
            println!("My name is {}", self.name);
        }
    }
}

extern "C" fn MyStruct_ctor0(_: usize, _: *mut u8) -> Ptr {
    let value = Box::new(MyStruct::new());
    Box::into_raw(value) as *mut u8
}

extern "C" fn MyStruct_dtor(value: Ptr) {
    unsafe {
        ptr::drop_in_place(value);
        dealloc(value, Layout::new::<MyStruct>());
    }
}

extern "C" fn MyStruct_repeat_name(args_size: usize, args: *mut u8) -> *mut u8 {
    unsafe {
        let args_slice = std::slice::from_raw_parts_mut(args, args_size);
        let this: *mut MyStruct = *(args_slice.as_mut_ptr() as *mut Ptr) as *mut MyStruct;
        let times: u32 = *(args_slice.as_mut_ptr().add(std::mem::size_of::<*mut MyStruct>()) as *mut u32);
        (*this).repeat_name(times);
    }
    ptr::null_mut()
}

#[test]
fn it_works() {
    let mut rt = Runtime::new();
    let mut ctx = rt.use_context();
    {
        // Store what we need
        let mut my_pkg = ctx.add_package(0);
        let mut MyStruct_type = my_pkg.add_type(0);
        MyStruct_type.add_ctor(MyStruct_ctor0);
        MyStruct_type.set_dtor(MyStruct_dtor);
        MyStruct_type.add_method(0, MyStruct_repeat_name);
    }
    {
        // Access what we need
        let my_pkg = ctx.get_package(0);
        let MyStruct_type = my_pkg.get_type(0);
        let ctor = MyStruct_type.get_ctor(0);
        let dtor = MyStruct_type.get_dtor();
        let repeat_name = MyStruct_type.get_method(0);

        // Create object, call the function and destroy the object
        let object = ctor(0, ptr::null_mut());
        let counter = 5u32;
        let args_size = std::mem::size_of::<Ptr>() + std::mem::size_of::<i32>();
        // Create buffer for storing arguments
        let mut args_buf = vec![0u8; args_size];
        let args_ptr = args_buf.as_mut_ptr();
        // Write the arguments to the pointer
        unsafe {
            *(args_ptr as *mut Ptr) = object;
            *(args_ptr.add(std::mem::size_of::<Ptr>()) as *mut u32) = counter;
        }
        repeat_name(args_size, args_ptr);
        dtor(object);
    }
}