use crate::{
    get_args,
    virtual_machine::{libs::lib::Library, value::Value, vm::VM},
};
use std::fs;

pub struct FSLib;

impl FSLib {
    fn read(_vm: &mut VM, args: Vec<Value>) -> Value {
        let path = get_args!(args);

        if let Value::String(path) = path {
            Value::string(fs::read_to_string(&*path.0).expect("Couldn't read file."))
        } else {
            panic!("`FS.read()` expects a string path")
        }
    }
}

// LIBRARY
impl Library for FSLib {
    fn get_name(&self) -> &str {
        "FS"
    }

    fn get_function(&self, name: u64) -> Box<dyn Fn(&mut VM, Vec<Value>) -> Value> {
        match name {
            // INPUT
            x if x == hash_u64!("read") => Box::new(Self::read),

            _ => panic!("Unknown function `{name}` on lib {}", self.get_name()),
        }
    }
}
