use std::alloc::{dealloc, Layout};
use std::ptr;
use tangara::context::{Property, Ptr};
use tangara::runtime::Runtime;

enum MyEnum {
    Unit,
    Vec3(f32, f32, f32)
}

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

    pub(crate) fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }
}

extern "C" fn MyEnum_Unit(_: usize, _: *mut u8) -> *mut u8 {
    let value = Box::new(MyEnum::Unit);
    Box::into_raw(value) as *mut u8
}

extern "C" fn MyEnum_Vec3(args_size: usize, args: *mut u8) -> *mut u8 {
    unsafe {
        let args_slice = std::slice::from_raw_parts_mut(args, args_size);
        let value1: f32 = ptr::read(args_slice.as_mut_ptr() as *const f32);
        let value2: f32 = ptr::read(args_slice.as_mut_ptr().add(std::mem::size_of::<*const f32>()) as *const f32);
        let value3: f32 = ptr::read(args_slice.as_mut_ptr().add(2*std::mem::size_of::<*const f32>()) as *const f32);
        let value = Box::new(MyEnum::Vec3(value1, value2, value3));
        Box::into_raw(value) as *mut u8
    }
}

extern "C" fn MyEnum_dtor(value: Ptr) {
    unsafe {
        ptr::drop_in_place(value);
        dealloc(value, Layout::new::<MyEnum>());
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
        let times: u32 = ptr::read(args_slice.as_mut_ptr().add(std::mem::size_of::<*mut MyStruct>()) as *mut u32);
        (*this).repeat_name(times);
    }
    ptr::null_mut()
}

extern "C" fn MyStruct_set_name(this: Ptr, object: Ptr) {
    unsafe {
        let this: *mut MyStruct = this as *mut MyStruct;
        let name: &str = ptr::read(object as *const &str);
        (*this).set_name(name);
    }
}

extern "C" fn MyStruct_get_name(this: Ptr) -> Ptr {
    unsafe {
        let this: *const MyStruct = this as *const MyStruct;
        let to_return = Box::new((*this).get_name());
        Box::into_raw(to_return) as Ptr
    }
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
        MyStruct_type.add_property(1, Property {
            getter: MyStruct_get_name,
            setter: Some(MyStruct_set_name)
        });
        let mut MyEnum_type = my_pkg.add_type(1);
        MyEnum_type.set_dtor(MyEnum_dtor);
        MyEnum_type.add_method(0, MyEnum_Unit);
        MyEnum_type.add_method(1, MyEnum_Vec3);
    }
    {
        // Access what we need
        let my_pkg = ctx.get_package(0);
        let MyStruct_type = my_pkg.get_type(0);
        let ctor = MyStruct_type.get_ctor(0);
        let dtor = MyStruct_type.get_dtor();
        let repeat_name = MyStruct_type.get_method(0);
        let name_property = MyStruct_type.get_property(1);

        // Create object
        let object = ctor(0, ptr::null_mut());

        // println!("I found the name {}", object.get_name());
        {
            let raw_ptr = (name_property.getter)(object);
            let name: &str = if !raw_ptr.is_null() {
                *unsafe {
                    Box::from_raw(raw_ptr as *mut &str)
                }
            } else {
                panic!("Pointer on name is null");
            };
            println!("I found the name {}", name);
        }

        // object.set_name("Alexander");
        {
            let name = "Alexander";
            // TODO: rewrite using ptr::read/write
            let name_boxed = Box::new(name);
            if let Some(set_name) = name_property.setter {
                set_name(object, Box::into_raw(name_boxed) as Ptr);
            }
        }

        /*
        // println!("I found the name {}", object.get_name());
        {
            let args_size = std::mem::size_of::<Ptr>();
            // Create buffer for storing arguments
            let mut args_buf = vec![0u8; args_size];
            let args_ptr = args_buf.as_mut_ptr();
            // Write the arguments to the pointer
            unsafe {
                *(args_ptr as *mut Ptr) = object;
            }
            let raw_ptr = get_name(args_size, args_ptr);
            let name: &str = if !raw_ptr.is_null() {
                *unsafe {
                    Box::from_raw(raw_ptr as *mut &str)
                }
            } else {
                panic!("Pointer on name is null");
            };
            println!("I found the name {}", name);
        }

        // object.set_name("Alexander");
        {
            let name = "Alexander";
            let args_size = std::mem::size_of::<Ptr>() + std::mem::size_of::<&str>();
            let mut args_buf = vec![0u8; args_size];
            let args_ptr = args_buf.as_mut_ptr();
            unsafe {
                *(args_ptr as *mut Ptr) = object;
                *(args_ptr.add(std::mem::size_of::<Ptr>()) as *mut &str) = name;
            }
            set_name(args_size, args_ptr);
        }
         */

        // object.repeat_name(5);
        {
            let counter = 5u32;
            let args_size = std::mem::size_of::<Ptr>() + std::mem::size_of::<i32>();
            let mut args_buf = vec![0u8; args_size];
            let args_ptr = args_buf.as_mut_ptr();
            unsafe {
                *(args_ptr as *mut Ptr) = object;
                *(args_ptr.add(std::mem::size_of::<Ptr>()) as *mut u32) = counter;
            }
            repeat_name(args_size, args_ptr);
        }

        // Destroy the object
        dtor(object);
    }
}