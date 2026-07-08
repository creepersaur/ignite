use bincode::{Decode, Encode};
use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Debug,
    hash::{Hash, Hasher},
    rc::Rc,
};

use crate::virtual_machine::{modules::Module, value::Value};

#[derive(Encode, Decode, Clone)]
pub struct TFunction {
    pub entry: usize,
    pub handler: Option<(u64, u64)>,
    pub this: Option<Box<Value>>,
    pub target: Option<Box<Value>>,
    pub upvalues: Vec<Rc<RefCell<HashMap<u64, (Value, bool)>>>>,
	pub module: Option<Rc<RefCell<Module>>>
}

impl TFunction {
    pub fn new(entry: usize, module: Rc<RefCell<Module>>) -> Self {
        Self {
            entry,
            handler: None,
            this: None,
            target: None,
            upvalues: vec![],
			module: Some(module.clone()),
        }
    }

    pub fn with_lib(lib: Rc<str>, method: Rc<str>, this: Option<Box<Value>>) -> Self {
        Self {
            entry: 0,
            handler: Some((hash_u64!(lib.as_ref()), hash_u64!(method.as_ref()))),
            target: None,
            this,
            upvalues: vec![],
			module: None,
        }
    }

    pub fn with_lib_id(lib: u64, method: u64, this: Option<Box<Value>>) -> Self {
        Self {
            entry: 0,
            handler: Some((lib, method)),
            target: None,
            this,
            upvalues: vec![],
			module: None,
        }
    }
}

impl Debug for TFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("entry: {}", self.entry)).unwrap();
        Ok(())
    }
}

impl Eq for TFunction {}

impl Hash for TFunction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.entry.hash(state);
        self.handler.hash(state);
        self.this.hash(state);
    }
}

impl PartialEq for TFunction {
    fn eq(&self, other: &Self) -> bool {
        self.entry == other.entry
    }
}

impl PartialOrd for TFunction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.entry.partial_cmp(&other.entry)
    }
}
