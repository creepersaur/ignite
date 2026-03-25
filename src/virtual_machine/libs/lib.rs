use std::rc::Rc;

use crate::virtual_machine::{value::Value, vm::VM};

pub trait Library {
    fn get_name(&self) -> &str;

    fn get_function(&self, name: Rc<String>) -> Box<dyn Fn(&mut VM) -> Value>;
}
