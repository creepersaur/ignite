use std::io::Read;

use crate::{
    get_args,
    virtual_machine::{
        libs::{lib::Library, namespaces::classes::file::file::FileData},
        types::list::TList,
        value::Value,
        vm::VM,
    },
};

pub struct FileLib;

impl FileLib {
    fn as_file_data(file: Value, panic_message: &str) -> FileData {
        if let Value::ClassObject(file) = file
            && let Some(data) = file.native_data
            && let Some(file_data) = data.as_any().downcast_ref::<FileData>().cloned()
        {
            file_data
        } else {
            panic!("{panic_message}")
        }
    }

    ///////////////////////////////////////////////
    /// Metadata
    ///////////////////////////////////////////////

    fn path(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [file] = get_args!(args, 1);

        let file_data = Self::as_file_data(file, "File.path() can only be used on Files");

        Value::string(file_data.path.to_str().unwrap())
    }

    fn name(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [file] = get_args!(args, 1);

        let file_data = Self::as_file_data(file, "File.name() can only be used on Files");

        Value::string(file_data.path.file_name().unwrap().to_str().unwrap())
    }

    fn extension(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [file] = get_args!(args, 1);

        let file_data = Self::as_file_data(file, "File.extension() can only be used on Files");

        Value::string(file_data.path.extension().unwrap().to_str().unwrap())
    }

    fn stem(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [file] = get_args!(args, 1);

        let file_data = Self::as_file_data(file, "File.stem() can only be used on Files");

        Value::string(file_data.path.file_stem().unwrap().to_str().unwrap())
    }

    fn size(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [file] = get_args!(args, 1);

        let file_data = Self::as_file_data(file, "File.size() can only be used on Files");

        if file_data.path.exists() {
            Value::Number(
                std::fs::metadata(file_data.path)
                    .expect("Could not get file size")
                    .len() as f64,
            )
        } else {
            panic!(
                "Tried getting size of non-existing file (`{:?}`)",
                file_data.path
            )
        }
    }

    fn exists(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [file] = get_args!(args, 1);

        let file_data = Self::as_file_data(file, "File.exists() can only be used on Files");

        Value::Bool(file_data.path.exists())
    }

    fn is_file(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [file] = get_args!(args, 1);

        let file_data = Self::as_file_data(file, "File.is_file() can only be used on Files");

        Value::Bool(file_data.path.is_file())
    }

    fn is_dir(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [file] = get_args!(args, 1);

        let file_data = Self::as_file_data(file, "File.is_dir() can only be used on Files");

        Value::Bool(file_data.path.is_dir())
    }

    // fn parent(_vm: &mut VM, args: Vec<Value>) -> Value {
    //     let [file] = get_args!(args, 1);

    //     let file_data = Self::as_file_data(file, "File.parent() can only be used on Files");

    // 	todo!("Add `File.parent()`")
    // }

    ///////////////////////////////////////////////
    // IO
    ///////////////////////////////////////////////

    fn read(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [file] = get_args!(args, 1);

        let file_data = Self::as_file_data(file, "File.read() can only be used on Files");

        if file_data.path.exists() {
            Value::string(std::fs::read_to_string(&file_data.path).expect("Could not read file"))
        } else {
            panic!("Tried reading non-existing file (`{:?}`)", file_data.path)
        }
    }

    fn read_bytes(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [file] = get_args!(args, 1);

        let file_data = Self::as_file_data(file, "File.read_bytes() can only be used on Files");

        if file_data.path.exists() {
            Value::List(TList::from(
                std::fs::read(&file_data.path)
                    .expect("Could not read file bytes")
                    .iter()
                    .map(|x| Value::Number(*x as f64))
                    .collect::<Vec<_>>(),
            ))
        } else {
            panic!("Tried reading non-existing file (`{:?}`)", file_data.path)
        }
    }

    fn read_exact(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [file, bytes] = get_args!(args, 2);

        let file_data = Self::as_file_data(file, "File.read_exact() can only be used on Files");

        if file_data.path.exists() {
            let mut buffer = vec![0; bytes.as_number("number convertion") as usize];

            std::fs::File::open(&file_data.path)
                .expect("Could not read file bytes")
                .read_exact(&mut buffer)
                .expect("Reached [EOF] before finishing reading exact");

            Value::List(TList::from(
                buffer
                    .iter()
                    .map(|x| Value::Number(*x as f64))
                    .collect::<Vec<_>>(),
            ))
        } else {
            panic!("Tried reading non-existing file (`{:?}`)", file_data.path)
        }
    }

    fn write(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [file, contents] = get_args!(args, 2);

        let file_data = Self::as_file_data(file, "File.write() can only be used on Files");

        std::fs::write(file_data.path, contents.as_str()).expect("Could not write to file");

        Value::NIL
    }
}

// Library
impl Library for FileLib {
    fn get_name(&self) -> &str {
        "File"
    }

    fn get_function(&self, name: u64) -> Box<dyn Fn(&mut VM, Vec<Value>) -> Value> {
        match name {
            // Metadata
            x if x == hash_u64!("path") => boxed!(Self::path),
            x if x == hash_u64!("name") => boxed!(Self::name),
            x if x == hash_u64!("extension") => boxed!(Self::extension),
            x if x == hash_u64!("stem") => boxed!(Self::stem),
            x if x == hash_u64!("is_file") => boxed!(Self::is_file),
            x if x == hash_u64!("is_dir") => boxed!(Self::is_dir),
            x if x == hash_u64!("exists") => boxed!(Self::exists),
            x if x == hash_u64!("size") => boxed!(Self::size),
            // x if x == hash_u64!("parent") => boxed!(Self::parent),

            // IO
            x if x == hash_u64!("read") => boxed!(Self::read),
            x if x == hash_u64!("read_bytes") => boxed!(Self::read_bytes),
            x if x == hash_u64!("read_exact") => boxed!(Self::read_exact),
            x if x == hash_u64!("write") => boxed!(Self::write),

            _ => panic!("Unknown function `{name}` on lib {}", self.get_name()),
        }
    }
}
