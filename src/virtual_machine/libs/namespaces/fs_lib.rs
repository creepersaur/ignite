use crate::{
    get_args,
    virtual_machine::{
        libs::{lib::Library, namespaces::classes::file::file::FileObject},
        value::Value,
        vm::VM,
    },
};

pub struct FSLib;

impl FSLib {
    fn get_file(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [path] = get_args!(args, 1);
        let file_obj = FileObject::new(path.as_str().into());

        Value::ClassObject(file_obj.class_object)
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
            x if x == hash_u64!("get_file") => boxed!(Self::get_file),

            _ => panic!("Unknown function `{name}` on lib {}", self.get_name()),
        }
    }
}
