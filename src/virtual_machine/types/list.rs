use std::{cell::RefCell, fmt::Debug, rc::Rc};

use crate::{
    misc::to_index::to_index,
    virtual_machine::{
        libs::types::{
            list_lib::{LIST_FUNCTION_IDS, LIST_FUNCTIONS},
            tuple_lib::{TUPLE_FUNCTION_IDS, TUPLE_FUNCTIONS},
        },
        traits::member_accessible::IMemberAccessible,
        types::function::TFunction,
        value::Value,
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
    pub fn empty() -> Self {
        Self {
            values: Rc::new(RefCell::new(vec![])),
            is_tuple: false,
        }
    }
}

impl From<Vec<Value>> for TList {
    fn from(value: Vec<Value>) -> Self {
        Self::new(Rc::new(RefCell::new(value)))
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

    fn get_member_id(&self, vm: &mut VM, member: &u64) -> Value {
        match self.is_tuple {
            true => {
                if TUPLE_FUNCTION_IDS.contains(member) {
                    return lib_function_id!(self, hash_u64!("tuple"), *member, Value::Tuple);
                }
            }
            false => {
                if LIST_FUNCTION_IDS.contains(member) {
                    return lib_function_id!(self, hash_u64!("list"), *member, Value::List);
                }
            }
        }

        panic!(
            "Cannot get member `{}` on {self:?}",
            vm.lookup_intern(*member)
        );
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

    fn set_member_id(&mut self, vm: &mut VM, member: &u64, _value: Value) {
        panic!(
            "Cannot set member `{}` on {self:?}",
            vm.lookup_intern(*member)
        );
    }
}
