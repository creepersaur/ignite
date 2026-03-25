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

pub const STRING_FUNCTIONS: [&str; 13] = [
    "len", "push", "insert", "remove", "pop", "clear", "concat", "copy", "count", "reverse",
    "fill", "rep", "push_n",
];

#[derive(Debug, Encode, Decode, Clone, PartialEq, PartialOrd)]
pub struct TString(pub Rc<RefCell<String>>);

// MEMBER ACCESS
impl IMemberAccessible for TString {
    fn get_member(&self, _vm: &mut VM, member: &Value) -> Value {
        if let Value::Number(index) = member {
            let len = self.0.borrow().len();
            let target_index = to_index(*index, len);

            return Value::String(TString(rc!(RefCell::new(
                self.0.borrow()[target_index..target_index + 1].to_string()
            ))));
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
