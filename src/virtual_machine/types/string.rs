use std::{cell::RefCell, fmt::Debug, rc::Rc};

use crate::{
    lib_function,
    misc::to_index::to_index,
    rc,
    virtual_machine::{
        traits::member_accessible::IMemberAccessible, types::function::TFunction, value::Value,
        vm::VM,
    },
};
use bincode::{Decode, Encode};

pub const STRING_FUNCTIONS: [&str; 15] = [
    "len", "push", "insert", "remove", "pop", "clear", "concat", "copy", "count", "reverse",
    "fill", "rep", "push_n", "chars", "bytes"
];

#[derive(Encode, Decode, Clone, PartialEq, PartialOrd)]
pub struct TString(pub Rc<RefCell<String>>);

impl TString {
    pub fn to_string(&self) -> String {
        self.0.borrow().clone()
    }
}

impl Debug for TString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("\"{}\"", self.0.borrow())).unwrap();
        Ok(())
    }
}

// MEMBER ACCESS
impl IMemberAccessible for TString {
    fn get_member(&self, _vm: &mut VM, member: &Value) -> Value {
        if let Value::Number(index) = member {
            let len = self.0.borrow().len();
            let target_index = to_index(*index, len);

            return Value::Char(self.0.borrow().chars().nth(target_index).unwrap());
        }

        if let Value::String(member) = member {
            if STRING_FUNCTIONS.contains(&member.0.borrow().as_str()) {
                return lib_function!(self, "string", member.0, 1, Value::String);
            }
        }

        panic!("Cannot get member `{}` on {self:?}", member.to_string(true));
    }

    fn set_member(&self, member: &Value, _value: Value) {
        panic!("Cannot set member `{}` on {self:?}", member.to_string(true));
    }
}
