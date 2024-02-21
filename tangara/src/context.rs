use std::collections::HashMap;

pub type Ptr = *mut u8;
pub type FnDtor = extern "C" fn(Ptr);
pub type Fn = extern "C" fn(usize, *mut u8) -> Ptr;

pub struct Property {
    pub getter: extern "C" fn(Ptr) -> Ptr,
    pub setter: Option<extern "C" fn(Ptr, Ptr)>
}
pub struct StaticProperty {
    pub getter: extern "C" fn() -> Ptr,
    pub setter: Option<extern "C" fn(Ptr)>
}

pub struct FuncTable {
    dtor: Option<FnDtor>,
    ctors: Vec<Fn>,
    methods: HashMap<u64, Fn>,
    properties: HashMap<u64, Property>,
    statics: HashMap<u64, StaticProperty>
}

impl FuncTable {
    pub(crate) fn new() -> Self {
        Self {
            dtor: None,
            ctors: Vec::new(),
            methods: HashMap::new(),
            properties: HashMap::new(),
            statics: HashMap::new()
        }
    }

    pub fn set_dtor(&mut self, dtor: FnDtor) {
        self.dtor = Some(dtor);
    }

    pub fn get_dtor(&self) -> FnDtor {
        self.dtor.expect("Destructor cannot be None on calling")
    }

    pub fn add_ctor(&mut self, ctor: Fn) -> usize {
        self.ctors.push(ctor);
        self.ctors.len() - 1
    }

    pub fn get_ctor(&self, index: usize) -> &Fn {
        self.ctors.get(index).expect(format!("Constructor not found at {index} index").as_str())
    }

    pub fn add_method(&mut self, id: u64, func: Fn) {
        self.methods.insert(id, func);
    }

    pub fn get_method(&self, id: u64) -> &Fn {
        self.methods.get(&id).expect(format!("Method with id {id} is not found").as_str())
    }

    pub fn add_property(&mut self, id: u64, property: Property) {
        self.properties.insert(id, property);
    }

    pub fn get_property(&self, id: u64) -> &Property {
        self.properties.get(&id).expect(format!("Property with id {id} is not found").as_str())
    }

    pub fn add_static(&mut self, id: u64, static_property: StaticProperty) {
        self.statics.insert(id, static_property);
    }

    pub fn get_static(&self, id: u64) -> &StaticProperty {
        self.statics.get(&id).expect(format!("Static property with id {id} is not found").as_str())
    }
}

pub struct TypeTable {
    types: HashMap<u64, FuncTable>
}

impl TypeTable {
    pub(crate) fn new() -> Self {
        Self {
            types: HashMap::new()
        }
    }

    pub fn add_type(&mut self, id: u64) -> &mut FuncTable {
        self.types.insert(id, FuncTable::new());
        self.types.get_mut(&id).unwrap()
    }

    pub fn get_type(&self, id: u64) -> &FuncTable {
        self.types.get(&id).expect(format!("Type by id {id} not found").as_str())
    }
}

pub struct Context {
    pkgs: HashMap<u64, TypeTable>
}

impl Context {
    pub(crate) fn new() -> Self {
        Self {
            pkgs: HashMap::new()
        }
    }

    pub fn add_package(&mut self, id: u64) -> &mut TypeTable {
        self.pkgs.insert(id, TypeTable::new());
        self.pkgs.get_mut(&id).unwrap()
    }

    pub fn get_package(&self, id: u64) -> &TypeTable {
        self.pkgs.get(&id).expect(format!("Package by id {id} not found").as_str())
    }
}