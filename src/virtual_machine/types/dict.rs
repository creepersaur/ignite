use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use crate::{
    lib_function, rc,
    virtual_machine::{
        traits::member_accessible::IMemberAccessible, types::function::TFunction, value::Value,
        vm::VM,
    },
};
use bincode::{Decode, Encode};

#[derive(Encode, Decode, Clone, PartialEq)]
pub struct TDict {
    pub values: Rc<RefCell<HashMap<Value, Value>>>,
}

impl PartialOrd for TDict {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

impl TDict {
    pub fn new(values: Rc<RefCell<HashMap<Value, Value>>>) -> Self {
        Self { values }
    }
}

impl Debug for TDict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Dict").unwrap();
        Ok(())
    }
}

// MEMBER ACCESS
impl IMemberAccessible for TDict {
    fn get_member(&self, _vm: &mut VM, member: &Value) -> Value {
        if let Value::String(member) = member {
            let functions = [
                "len", "items", "keys", "values", "get", "insert", "remove", "clear", "append",
                "concat", "count",
            ];

            if functions.contains(&member.as_str()) {
                return lib_function!(self, "dict", member, 1, Value::Dict);
            }
        }

        if let Some(x) = self.values.borrow().get(member) {
            return x.clone();
        }

        panic!("Cannot get member `{}` on {self:?}", member.to_string(true));
    }

    fn set_member(&self, member: &Value, value: Value) {
        self.values.borrow_mut().insert(member.clone(), value);

        // panic!("Cannot set member `{}` on {self:?}", member.to_string(true));
    }
}
