use crate::virtual_machine::{
    types::{
        classes::{class::TClass, class_object::TClassObject},
        function::TFunction,
    },
    value::Value,
};
use std::{cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc};

thread_local! {
    static DIR_CLASS: Rc<RefCell<TClass>> = Rc::new(RefCell::new(TClass {
        name: "Directory".into(),
        values: rc!(RefCell::new(HashMap::new())),
        functions: rc!(RefCell::new({
            let mut map = HashMap::new();

            // Metadata
            map.insert(hash_u64!("path"), lib_function_id!("Directory", "path"));
            map.insert(hash_u64!("name"), lib_function_id!("Directory", "name"));
            map.insert(hash_u64!("parent"), lib_function_id!("Directory", "parent"));
            map.insert(hash_u64!("exists"), lib_function_id!("Directory", "exists"));
            map.insert(hash_u64!("is_dir"), lib_function_id!("Directory", "is_dir"));
            map.insert(hash_u64!("is_file"), lib_function_id!("Directory", "is_file"));

            // Filesystem
            map.insert(hash_u64!("rename"), lib_function_id!("Directory", "rename"));
            map.insert(hash_u64!("move"), lib_function_id!("Directory", "move"));
            map.insert(hash_u64!("create"), lib_function_id!("Directory", "create"));
            map.insert(hash_u64!("create_all"), lib_function_id!("Directory", "create_all"));
            map.insert(hash_u64!("read"), lib_function_id!("Directory", "read"));
            map.insert(hash_u64!("delete"), lib_function_id!("Directory", "delete"));
            map.insert(hash_u64!("delete_all"), lib_function_id!("Directory", "delete_all"));

            map
        })),
        constructor: None,
    }));
}

#[derive(Clone)]
pub struct DirectoryObject {
    pub class_object: TClassObject,
}

#[derive(Clone)]
pub struct DirectoryData {
    pub path: Rc<RefCell<PathBuf>>,
}

impl DirectoryObject {
    pub fn new(path: PathBuf) -> Self {
        let path = std::env::current_dir()
            .expect("Failed to get current directory")
            .join(path);

        Self {
            class_object: TClassObject::with_native(
                DIR_CLASS.with(Rc::clone),
                DirectoryData {
                    path: rc!(RefCell::new(path)),
                },
            ),
        }
    }

    #[allow(unused)]
    pub fn new_as_classobject(path: PathBuf) -> TClassObject {
        Self::new(path).class_object
    }
}
