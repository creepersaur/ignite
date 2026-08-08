use crate::{
    get_args,
    virtual_machine::{
        libs::{
            lib::Library,
            namespaces::classes::{directory::directory::DirectoryObject, file::file::FileObject},
        },
        value::Value,
        vm::VM,
    },
};

pub struct FSLib;

impl FSLib {
    fn get_file(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [path] = get_args!(args, 1);
        let file_obj = FileObject::new(path.as_str());

        Value::ClassObject(file_obj.class_object)
    }

    fn get_dir(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [path] = get_args!(args, 1);
        let file_obj = DirectoryObject::new(path.as_str());

        Value::ClassObject(file_obj.class_object)
    }
}

// LIBRARY
impl Library for FSLib {
    fn get_name(&self) -> &str {
        "FS"
    }

    fn get_function(&self, name: u64) -> Option<Box<dyn Fn(&mut VM, Vec<Value>) -> Value>> {
        Some(match name {
            // INPUT
            x if x == hash_u64!("get_file") => boxed!(Self::get_file),
            x if x == hash_u64!("get_dir") => boxed!(Self::get_dir),

            _ => return None
        })
    }
}
