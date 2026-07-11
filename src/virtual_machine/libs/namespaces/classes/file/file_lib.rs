use std::{
    io::{Read, Write},
    path::PathBuf,
};

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

    fn prefix(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [file] = get_args!(args, 1);

        let file_data = Self::as_file_data(file, "File.prefix() can only be used on Files");

        Value::string(file_data.path.file_prefix().unwrap().to_str().unwrap())
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

    fn rename(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [file, new_name] = get_args!(args, 2);

        let file_data = Self::as_file_data(file, "File.rename() can only be used on Files");

        if file_data.path.exists() {
            std::fs::rename(file_data.path, new_name.as_str()).expect("Could not rename file");

            Value::NIL
        } else {
            panic!("Tried renaming non-existing file (`{:?}`)", file_data.path)
        }
    }

    fn r#move(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [file, destination] = get_args!(args, 2);

        let file_data = Self::as_file_data(file, "File.move() can only be used on Files");

        if file_data.path.exists() {
            let path_buf =
                PathBuf::from(destination.as_str()).join(file_data.path.file_name().unwrap());
            std::fs::rename(file_data.path, path_buf).expect("Could not move file");

            Value::NIL
        } else {
            panic!("Tried moving non-existing file (`{:?}`)", file_data.path)
        }
    }

    fn copy(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [file, new_name] = get_args!(args, 2);

        let file_data = Self::as_file_data(file, "File.copy() can only be used on Files");

        if file_data.path.exists() {
            std::fs::copy(file_data.path, new_name.as_str()).expect("Could not copy file");

            Value::NIL
        } else {
            panic!("Tried copying non-existing file (`{:?}`)", file_data.path)
        }
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

    fn write_bytes(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [file, contents] = get_args!(args, 2);

        let file_data = Self::as_file_data(file, "File.write_bytes() can only be used on Files");

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&file_data.path)
            .expect("Could not open file for writing bytes");

        if let Value::List(TList { values, .. }) = contents {
            file.write(
                &values
                    .borrow()
                    .iter()
                    .map(|x| x.as_number("number convertion") as u8)
                    .collect::<Vec<_>>(),
            )
            .expect("Could not append bytes to file");
        } else {
            panic!("Expected list of bytes (numbers), got {contents:?}")
        }

        Value::NIL
    }

    fn append(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [file, contents] = get_args!(args, 2);

        let file_data = Self::as_file_data(file, "File.append() can only be used on Files");

        if file_data.path.exists() {
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(file_data.path)
                .expect("Could not open file for appending")
                .write_all(contents.as_str().as_bytes())
                .expect("Could not append to file");
        } else {
            panic!(
                "Tried appending to non-existing file (`{:?}`)",
                file_data.path
            )
        }

        Value::NIL
    }

    fn append_bytes(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [file, contents] = get_args!(args, 2);

        let file_data = Self::as_file_data(file, "File.append_bytes() can only be used on Files");

        if file_data.path.exists() {
            if let Value::List(TList { values, .. }) = contents {
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(file_data.path)
                    .expect("Could not open file for appending")
                    .write_all(
                        &values
                            .borrow()
                            .iter()
                            .map(|x| x.as_number("number convertion") as u8)
                            .collect::<Vec<_>>(),
                    )
                    .expect("Could not append bytes to file");
            }
        } else {
            panic!(
                "Tried appending bytes to non-existing file (`{:?}`)",
                file_data.path
            )
        }

        Value::NIL
    }

    fn delete(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [file] = get_args!(args, 1);

        let file_data = Self::as_file_data(file, "File.delete() can only be used on Files");

        if file_data.path.exists() {
            std::fs::remove_file(file_data.path).expect("Could not delete file");
        } else {
            panic!("Tried deleting non-existing file (`{:?}`)", file_data.path)
        }

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
            x if x == hash_u64!("prefix") => boxed!(Self::prefix),
            x if x == hash_u64!("is_file") => boxed!(Self::is_file),
            x if x == hash_u64!("is_dir") => boxed!(Self::is_dir),
            x if x == hash_u64!("exists") => boxed!(Self::exists),
            x if x == hash_u64!("size") => boxed!(Self::size),
            x if x == hash_u64!("rename") => boxed!(Self::rename),
            x if x == hash_u64!("move") => boxed!(Self::r#move),
            x if x == hash_u64!("copy") => boxed!(Self::copy),
            // x if x == hash_u64!("parent") => boxed!(Self::parent),

            // IO
            x if x == hash_u64!("read") => boxed!(Self::read),
            x if x == hash_u64!("read_bytes") => boxed!(Self::read_bytes),
            x if x == hash_u64!("read_exact") => boxed!(Self::read_exact),
            x if x == hash_u64!("write") => boxed!(Self::write),
            x if x == hash_u64!("write_bytes") => boxed!(Self::write_bytes),
            x if x == hash_u64!("append") => boxed!(Self::append),
            x if x == hash_u64!("append_bytes") => boxed!(Self::append_bytes),
            x if x == hash_u64!("delete") => boxed!(Self::delete),

            _ => panic!("Unknown function `{name}` on lib {}", self.get_name()),
        }
    }
}
