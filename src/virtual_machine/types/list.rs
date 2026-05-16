use std::{cell::RefCell, fmt::Debug, rc::Rc};

use crate::{
    misc::to_index::to_index,
    virtual_machine::{
        libs::types::list_lib::LIST_FUNCTIONS, libs::types::tuple_lib::TUPLE_FUNCTIONS,
        traits::member_accessible::IMemberAccessible, types::function::TFunction, value::Value,
        vm::VM,
    },
};
use bincode::{Decode, Encode};

#[derive(Encode, Decode, Clone, PartialEq, PartialOrd)]
pub struct TList {
    pub values: Rc<RefCell<Vec<Value>>>,
    pub is_tuple: bool,
}

impl TList {
    pub fn new(values: Rc<RefCell<Vec<Value>>>) -> Self {
        Self {
            values,
            is_tuple: false,
        }
    }
    pub fn new_tuple(values: Rc<RefCell<Vec<Value>>>) -> Self {
        Self {
            values,
            is_tuple: true,
        }
    }
}

impl Debug for TList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("List").unwrap();
        Ok(())
    }
}

// MEMBER ACCESS
impl IMemberAccessible for TList {
    fn get_member(&self, _vm: &mut VM, member: &Value) -> Value {
        if let Value::Number(index) = member {
            let len = self.values.borrow().len();
            let target_index = to_index(*index, len);

            return self.values.borrow()[target_index].clone();
        }

        if let Value::String(member) = member {
            match self.is_tuple {
                true => {
                    if TUPLE_FUNCTIONS.contains(&&*member.0) {
                        return lib_function!(self, "tuple", member.0.clone(), Value::Tuple);
                    }
                }
                false => {
                    if LIST_FUNCTIONS.contains(&&*member.0) {
                        return lib_function!(self, "list", member.0.clone(), Value::List);
                    }
                }
            }
        }

        panic!("Cannot get member `{}` on {self:?}", member.to_string(true));
    }

    fn set_member(&mut self, member: &Value, value: Value) {
        if let Value::Number(index) = member {
            let len = self.values.borrow().len();
            let target_index = to_index(*index, len);

            self.values.borrow_mut()[target_index] = value;
            return;
        }

        panic!("Cannot set member `{}` on {self:?}", member.to_string(true));
    }
}
