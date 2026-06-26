use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use crate::virtual_machine::{
    libs::types::dict_lib::DICT_FUNCTION_IDS, traits::member_accessible::IMemberAccessible,
    types::function::TFunction, value::Value, vm::VM,
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
        if let Some(x) = self.values.borrow().get(member) {
            return x.clone();
        }

        panic!("Cannot get member `{}` on {self:?}", member.to_string(true));
    }

    fn set_member(&mut self, member: &Value, value: Value) {
        self.values.borrow_mut().insert(member.clone(), value);

        // panic!("Cannot set member `{}` on {self:?}", member.to_string(true));
    }

    fn get_member_id(&self, vm: &mut VM, member: &u64) -> Value {
        if DICT_FUNCTION_IDS.contains(member) {
            return lib_function_id!(self, hash_u64!("dict"), *member, Value::Dict);
        }

        panic!(
            "Cannot get member id `{}` on {self:?}",
            vm.lookup_intern(*member)
        );
    }
}
