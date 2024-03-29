// This file was generated by tangara-gen
// All changes in this file will discard after rebuilding project
use tangara::context::{FnDtor, Context, Ptr, Fn};

static mut MyStruct_ctor0: Option<Fn> = None;
static mut MyStruct_repeat_name: Option<Fn> = None;
static mut MyStruct_set_name: Option<Fn> = None;
static mut MyStruct_get_name: Option<Fn> = None;
static mut MyStruct_dtor: Option<FnDtor> = None;
static mut TestStruct_ctor0: Option<Fn> = None;
static mut TestStruct_ctor1: Option<Fn> = None;
static mut TestStruct_id_getter: Option<extern "C" fn(Ptr) -> Ptr> = None;
static mut TestStruct_id_setter: Option<extern "C" fn(Ptr, Ptr)> = None;
static mut TestStruct_dtor: Option<FnDtor> = None;

#[derive(Ord, PartialOrd, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum EnumUnit {
	Variant = 0,
}

pub enum EnumTuple {
}

pub enum EnumStruct {
}

pub enum EnumMixed {
}

pub enum EnumComplex {
}

pub trait MyTrait {
	fn foo(&mut self, a:String);
	fn bar(&self) -> String;
}

pub type BoxedStr = Box<str>;

pub struct MyStruct {
    ptr: Ptr
}

impl MyStruct {
	pub fn new(name:&str) -> Self {
		unsafe {
			if let Some(ctor_func) = MyStruct_ctor0 {
                let args_size = std::mem::size_of::<&str>();
                let mut args_buf = vec![0u8; args_size];
                let args_ptr = args_buf.as_mut_ptr();
                unsafe {
                    *(args_ptr as *mut &str) = name;
                }
                let this = ctor_func(args_size, args_ptr);
                if !this.is_null() {
                    Self {
                        ptr: this
                    }
                } else {
                    panic!("Pointer of constructor result is null")
                }
            }
            else {
                panic!("Constructor wasn't loaded")
            }
        }
    }
	pub fn repeat_name(&self, times:u32) -> () {
		unsafe {
			if let Some(method_func) = MyStruct_repeat_name {
                let args_size = std::mem::size_of::<Ptr>() + std::mem::size_of::<u32>();
                let mut args_buf = vec![0u8; args_size];
                let args_ptr = args_buf.as_mut_ptr();
                unsafe {
                    *(args_ptr as *mut Ptr) = self.ptr;
					*(args_ptr.add(std::mem::size_of::<Ptr>()) as *mut u32) = times;
                }
                let raw_ptr = method_func(args_size, args_ptr);
                if !raw_ptr.is_null() {
                    *Box::from_raw(raw_ptr as *mut ())
                } else {
                    panic!("Pointer of method result is null")
                }
            }
            else {
                panic!("Constructor wasn't loaded")
            }
        }
    }
	pub fn set_name(&mut self, name:&str) {
		unsafe {
			if let Some(method_func) = MyStruct_set_name {
                let args_size = std::mem::size_of::<Ptr>() + std::mem::size_of::<&str>();
                let mut args_buf = vec![0u8; args_size];
                let args_ptr = args_buf.as_mut_ptr();
                unsafe {
                    *(args_ptr as *mut Ptr) = self.ptr;
					*(args_ptr.add(std::mem::size_of::<Ptr>()) as *mut &str) = name;
                }
				method_func(args_size, args_ptr);
            }
            else {
                panic!("Constructor wasn't loaded")
            }
        }
    }
	pub fn get_name(&self) -> &str {
		unsafe {
			if let Some(method_func) = MyStruct_get_name {
                let args_size = std::mem::size_of::<Ptr>();
                let mut args_buf = vec![0u8; args_size];
                let args_ptr = args_buf.as_mut_ptr();
                unsafe {
                    *(args_ptr as *mut Ptr) = self.ptr;
                }
                let raw_ptr = method_func(args_size, args_ptr);
                if !raw_ptr.is_null() {
                    *Box::from_raw(raw_ptr as *mut &str)
                } else {
                    panic!("Pointer of method result is null")
                }
            }
            else {
                panic!("Constructor wasn't loaded")
            }
        }
    }
}

impl Drop for MyStruct {
	fn drop(&mut self) {
		unsafe {
			MyStruct_dtor.expect("Destructor wasn't loaded from library")(self.ptr);
		}
	}
}

pub struct TestStruct {
    ptr: Ptr
}

impl TestStruct {
	pub fn test_empty_ctor() -> Self {
		unsafe {
			if let Some(ctor_func) = TestStruct_ctor0 {
                let this = ctor_func(0, std::ptr::null_mut());
                if !this.is_null() {
                    Self {
                        ptr: this
                    }
                } else {
                    panic!("Pointer of constructor result is null")
                }
            }
            else {
                panic!("Constructor wasn't loaded")
            }
        }
    }
	pub fn new(id:u64) -> Self {
		unsafe {
			if let Some(ctor_func) = TestStruct_ctor1 {
                let args_size = std::mem::size_of::<u64>();
                let mut args_buf = vec![0u8; args_size];
                let args_ptr = args_buf.as_mut_ptr();
                unsafe {
                    *(args_ptr as *mut u64) = id;
                }
                let this = ctor_func(args_size, args_ptr);
                if !this.is_null() {
                    Self {
                        ptr: this
                    }
                } else {
                    panic!("Pointer of constructor result is null")
                }
            }
            else {
                panic!("Constructor wasn't loaded")
            }
        }
    }
	pub fn get_id(&self) -> u64 {
		unsafe {
			let raw_ptr: *mut u64 = TestStruct_id_getter.unwrap()(self.ptr) as *mut u64;
			if !raw_ptr.is_null() {
				*Box::from_raw(raw_ptr)
			} else {
				panic!("Pointer of gotten property is null")
			}
		}
	}
	pub fn set_id(&mut self, value: u64) {
		unsafe { TestStruct_id_setter.unwrap()(self.ptr, &value as *const u64 as Ptr); }
	}
}

impl Drop for TestStruct {
	fn drop(&mut self) {
		unsafe {
			TestStruct_dtor.expect("Destructor wasn't loaded from library")(self.ptr);
		}
	}
}

impl Default for TestStruct {
	fn default() -> Self {
		unsafe {
			TestStruct::test_empty_ctor()
		}
	}
}


pub fn load_mylib(ctx: &Context) {
	unsafe {
		let mylib_package = ctx.get_package(14252210530948059848);
		let EnumTuple_type = mylib_package.get_type(5703501090477233855);
		let EnumStruct_type = mylib_package.get_type(4061653529057324328);
		let EnumMixed_type = mylib_package.get_type(6533684593556827468);
		let EnumComplex_type = mylib_package.get_type(5514888211111417365);
		let MyStruct_type = mylib_package.get_type(11184697179514631841);
		MyStruct_ctor0 = Some(MyStruct_type.get_ctor(0).clone());
		MyStruct_repeat_name = Some(MyStruct_type.get_method(17567713076779176127).clone());
		MyStruct_set_name = Some(MyStruct_type.get_method(1641961565049420977).clone());
		MyStruct_get_name = Some(MyStruct_type.get_method(552281434682100053).clone());
		MyStruct_dtor = Some(MyStruct_type.get_dtor());
		let TestStruct_type = mylib_package.get_type(5562349104188291914);
		TestStruct_ctor0 = Some(TestStruct_type.get_ctor(0).clone());
		TestStruct_ctor1 = Some(TestStruct_type.get_ctor(1).clone());
		let TestStruct_id_prop = TestStruct_type.get_property(5824848936401749885);
		TestStruct_id_getter = Some(TestStruct_id_prop.getter);
		TestStruct_id_setter = Some(TestStruct_id_prop.setter.unwrap());
		TestStruct_dtor = Some(TestStruct_type.get_dtor());
	}
}
