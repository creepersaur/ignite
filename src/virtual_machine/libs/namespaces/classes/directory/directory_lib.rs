use crate::{
    get_args,
    virtual_machine::{
        libs::{
            lib::Library,
            namespaces::classes::{
                directory::directory::{DirectoryData, DirectoryObject},
                file::file::FileObject,
            },
        },
        types::list::TList,
        value::Value,
        vm::VM,
    },
};
use std::{fs, path::Path};

pub struct DirectoryLib;

impl DirectoryLib {
    fn as_directory_data(directory: Value, panic_message: &str) -> DirectoryData {
        if let Value::ClassObject(directory) = directory
            && let Some(data) = directory.native_data
            && let Some(directory_data) = data.as_any().downcast_ref::<DirectoryData>().cloned()
        {
            directory_data
        } else {
            panic!("{panic_message}")
        }
    }

    fn get_dir_files(path: impl AsRef<Path>) -> Vec<std::path::PathBuf> {
        fs::read_dir(path)
            .expect("Could not read directory")
            .filter_map(|entry| {
                let path = entry.ok()?.path();

                if path.is_file() { Some(path) } else { None }
            })
            .collect()
    }

    fn get_dir_dirs(path: impl AsRef<Path>) -> Vec<std::path::PathBuf> {
        fs::read_dir(path)
            .expect("Could not read directory")
            .filter_map(|entry| {
                let path = entry.ok()?.path();

                if path.is_dir() { Some(path) } else { None }
            })
            .collect()
    }

    fn get_dir_children(path: impl AsRef<Path>) -> Vec<std::path::PathBuf> {
        fs::read_dir(path)
            .expect("Could not read directory")
            .filter_map(|entry| Some(entry.ok()?.path()))
            .collect()
    }

    ///////////////////////////////////////////////
    /// Metadata
    ///////////////////////////////////////////////

    fn path(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [directory] = get_args!(args, 1);

        let directory_data = Self::as_directory_data(
            directory,
            "Directory.path() can only be used on Directories",
        );

        Value::string(directory_data.path.borrow().to_str().unwrap_or(""))
    }

    fn name(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [directory] = get_args!(args, 1);

        let directory_data = Self::as_directory_data(
            directory,
            "Directory.name() can only be used on Directories",
        );

        directory_data
            .path
            .borrow()
            .canonicalize()
            .expect("Failed to canonicalize path")
            .file_name()
            .and_then(|name| name.to_str())
            .map(Value::string)
            .unwrap_or(Value::NIL)
    }

    fn exists(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [directory] = get_args!(args, 1);

        let directory_data = Self::as_directory_data(
            directory,
            "Directory.exists() can only be used on Directories",
        );

        Value::Bool(directory_data.path.borrow().exists())
    }

    fn is_dir(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [directory] = get_args!(args, 1);

        let directory_data = Self::as_directory_data(
            directory,
            "Directory.is_dir() can only be used on Directories",
        );

        Value::Bool(directory_data.path.borrow().is_dir())
    }

    fn is_file(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [directory] = get_args!(args, 1);

        let directory_data = Self::as_directory_data(
            directory,
            "Directory.is_file() can only be used on Directories",
        );

        Value::Bool(directory_data.path.borrow().is_file())
    }

    fn parent(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [directory] = get_args!(args, 1);

        let directory_data = Self::as_directory_data(
            directory,
            "Directory.parent() can only be used on Directories",
        );

        directory_data
            .path
            .borrow()
            .parent()
            .and_then(|path| {
                Some(Value::ClassObject(DirectoryObject::new_as_classobject(
                    path,
                )))
            })
            .unwrap_or(Value::NIL)
    }

    fn get_children(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [directory] = get_args!(args, 1);

        let directory_data = Self::as_directory_data(
            directory,
            "Directory.get_children() can only be used on Directories",
        );

        let children = Self::get_dir_children(directory_data.path.borrow().as_path());

        Value::List(TList::from(
            children
                .iter()
                .map(|path| {
                    if path.is_file() {
                        Value::ClassObject(FileObject::new_as_classobject(path))
                    } else {
                        Value::ClassObject(DirectoryObject::new_as_classobject(path))
                    }
                })
                .collect::<Vec<_>>(),
        ))
    }

    fn get_files(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [directory] = get_args!(args, 1);

        let directory_data = Self::as_directory_data(
            directory,
            "Directory.get_files() can only be used on Directories",
        );

        let children = Self::get_dir_files(directory_data.path.borrow().as_path());

        Value::List(TList::from(
            children
                .iter()
                .map(|path| Value::ClassObject(FileObject::new_as_classobject(path)))
                .collect::<Vec<_>>(),
        ))
    }

    fn get_dirs(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [directory] = get_args!(args, 1);

        let directory_data = Self::as_directory_data(
            directory,
            "Directory.get_dirs() can only be used on Directories",
        );

        let children = Self::get_dir_dirs(directory_data.path.borrow().as_path());

        Value::List(TList::from(
            children
                .iter()
                .map(|path| Value::ClassObject(DirectoryObject::new_as_classobject(path)))
                .collect::<Vec<_>>(),
        ))
    }

    ///////////////////////////////////////////////
    /// Filesystem
    ///////////////////////////////////////////////

    fn rename(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [directory, new_name] = get_args!(args, 2);

        let directory_data = Self::as_directory_data(
            directory,
            "Directory.rename() can only be used on Directories",
        );

        if directory_data.path.borrow().exists() {
            let new_path = directory_data
                .path
                .borrow()
                .parent()
                .expect("Could not get directory parent")
                .join(new_name.as_str());

            fs::rename(directory_data.path.borrow().as_path(), &new_path)
                .expect("Could not rename directory");

            *directory_data.path.borrow_mut() = new_path;

            Value::NIL
        } else {
            panic!(
                "Tried renaming non-existing directory (`{:?}`)",
                directory_data.path
            )
        }
    }

    fn r#move(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [directory, destination] = get_args!(args, 2);

        let directory_data = Self::as_directory_data(
            directory,
            "Directory.move() can only be used on Directories",
        );

        if directory_data.path.borrow().exists() {
            let new_path = std::env::current_dir()
                .expect("Failed to get current directory")
                .join(destination.as_str())
                .join(
                    directory_data
                        .path
                        .borrow()
                        .file_name()
                        .expect("Could not get directory name while moving"),
                );

            fs::rename(directory_data.path.borrow().as_path(), &new_path)
                .expect("Could not move directory");

            *directory_data.path.borrow_mut() = new_path;

            Value::NIL
        } else {
            panic!(
                "Tried moving non-existing directory (`{:?}`)",
                directory_data.path
            )
        }
    }

    fn create(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [directory] = get_args!(args, 1);

        let directory_data = Self::as_directory_data(
            directory,
            "Directory.create() can only be used on Directories",
        );

        if directory_data.path.borrow().exists() {
            return Value::NIL;
        }

        fs::create_dir(directory_data.path.borrow().as_path()).expect("Could not create directory");

        Value::NIL
    }

    fn create_all(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [directory] = get_args!(args, 1);

        let directory_data = Self::as_directory_data(
            directory,
            "Directory.create_all() can only be used on Directories",
        );

        fs::create_dir_all(directory_data.path.borrow().as_path())
            .expect("Could not create directory and its parents");

        Value::NIL
    }

    fn read(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [directory] = get_args!(args, 1);

        let directory_data = Self::as_directory_data(
            directory,
            "Directory.read() can only be used on Directories",
        );

        if !directory_data.path.borrow().exists() {
            panic!(
                "Cannot read directory: directory does not exist (`{:?}`)",
                directory_data.path
            );
        }

        if !directory_data.path.borrow().is_dir() {
            panic!(
                "Cannot read directory: path is not a directory (`{:?}`)",
                directory_data.path
            );
        }

        let entries =
            fs::read_dir(directory_data.path.borrow().as_path()).expect("Could not read directory");

        Value::List(TList::from(
            entries
                .filter_map(|entry| {
                    entry
                        .ok()
                        .and_then(|entry| entry.path().to_str().map(Value::string))
                })
                .collect::<Vec<_>>(),
        ))
    }

    fn delete(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [directory] = get_args!(args, 1);

        let directory_data = Self::as_directory_data(
            directory,
            "Directory.delete() can only be used on Directories",
        );

        if !directory_data.path.borrow().exists() {
            panic!(
                "Cannot delete directory: directory does not exist (`{:?}`)",
                directory_data.path
            );
        }

        fs::remove_dir(directory_data.path.borrow().as_path()).expect("Could not delete directory");

        Value::NIL
    }

    fn delete_all(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [directory] = get_args!(args, 1);

        let directory_data = Self::as_directory_data(
            directory,
            "Directory.delete_all() can only be used on Directories",
        );

        if !directory_data.path.borrow().exists() {
            panic!(
                "Cannot delete directory: directory does not exist (`{:?}`)",
                directory_data.path
            );
        }

        fs::remove_dir_all(directory_data.path.borrow().as_path())
            .expect("Could not delete directory and its contents");

        Value::NIL
    }
}

// Library
impl Library for DirectoryLib {
    fn get_name(&self) -> &str {
        "Directory"
    }

    fn get_function(&self, name: u64) -> Option<Box<dyn Fn(&mut VM, Vec<Value>) -> Value>> {
        Some(match name {
            // Metadata
            x if x == hash_u64!("path") => boxed!(Self::path),
            x if x == hash_u64!("name") => boxed!(Self::name),
            x if x == hash_u64!("parent") => boxed!(Self::parent),
            x if x == hash_u64!("exists") => boxed!(Self::exists),
            x if x == hash_u64!("is_file") => boxed!(Self::is_file),
            x if x == hash_u64!("is_dir") => boxed!(Self::is_dir),
            x if x == hash_u64!("get_files") => boxed!(Self::get_files),
            x if x == hash_u64!("get_dirs") => boxed!(Self::get_dirs),
            x if x == hash_u64!("get_children") => boxed!(Self::get_children),

            // Filesystem
            x if x == hash_u64!("rename") => boxed!(Self::rename),
            x if x == hash_u64!("move") => boxed!(Self::r#move),
            x if x == hash_u64!("create") => boxed!(Self::create),
            x if x == hash_u64!("create_all") => boxed!(Self::create_all),
            x if x == hash_u64!("read") => boxed!(Self::read),
            x if x == hash_u64!("delete") => boxed!(Self::delete),
            x if x == hash_u64!("delete_all") => boxed!(Self::delete_all),

            _ => return None,
        })
    }
}
