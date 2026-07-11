use crate::virtual_machine::{
    types::{
        classes::{class::TClass, class_object::TClassObject},
        function::TFunction,
    },
    value::Value,
};
use std::{cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc};

thread_local! {
    static FILE_CLASS: Rc<RefCell<TClass>> = Rc::new(RefCell::new(TClass {
        name: "File".into(),
        values: rc!(RefCell::new(HashMap::new())),
        functions: rc!(RefCell::new({
            let mut map = HashMap::new();

			// Metadata
            map.insert(hash_u64!("exists"), lib_function_id!("File", "exists"));
            map.insert(hash_u64!("path"), lib_function_id!("File", "path"));
            map.insert(hash_u64!("name"), lib_function_id!("File", "name"));
            map.insert(hash_u64!("extension"), lib_function_id!("File", "extension"));
            map.insert(hash_u64!("stem"), lib_function_id!("File", "stem"));
            map.insert(hash_u64!("is_file"), lib_function_id!("File", "is_file"));
            map.insert(hash_u64!("is_dir"), lib_function_id!("File", "is_dir"));
            map.insert(hash_u64!("parent"), lib_function_id!("File", "parent"));

			// IO
			map.insert(hash_u64!("read"), lib_function_id!("File", "read"));
			map.insert(hash_u64!("read_bytes"), lib_function_id!("File", "read_bytes"));
			map.insert(hash_u64!("read_exact"), lib_function_id!("File", "read_exact"));
            map.insert(hash_u64!("write"), lib_function_id!("File", "write"));

            map
        })),
        constructor: None,
    }));
}

#[derive(Clone)]
pub struct FileObject {
    pub class_object: TClassObject,
}

#[derive(Clone)]
pub struct FileData {
    pub path: PathBuf,
}

impl FileObject {
    pub fn new(path: PathBuf) -> Self {
        Self {
            class_object: TClassObject::with_native(FILE_CLASS.with(Rc::clone), FileData { path }),
        }
    }
}
