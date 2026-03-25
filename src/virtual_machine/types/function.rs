use bincode::{Decode, Encode};
use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    rc::Rc,
};

use crate::virtual_machine::value::Value;

#[derive(Encode, Decode, Clone, PartialEq, PartialOrd)]
pub struct TFunction {
    pub entry: usize,
    pub args: usize,
    pub handler: Option<(Rc<String>, Rc<String>)>,
    pub this: Option<Box<Value>>,
}

impl TFunction {
    pub fn new(entry: usize, args: usize) -> Self {
        Self {
            entry,
            args,
            handler: None,
            this: None,
        }
    }
    pub fn with_lib(
        lib: Rc<String>,
        method: Rc<String>,
        args: usize,
        this: Option<Box<Value>>,
    ) -> Self {
        Self {
            entry: 0,
            args,
            handler: Some((lib, method)),
            this,
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
        self.args.hash(state);
        self.handler.hash(state);
        self.this.hash(state);
    }
}
