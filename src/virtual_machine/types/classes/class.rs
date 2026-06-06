use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use crate::virtual_machine::{traits::member_accessible::IMemberAccessible, value::Value, vm::VM};
use bincode::{Decode, Encode};

#[derive(Encode, Decode, Clone, PartialEq)]
pub struct TClass {
    pub name: Rc<str>,
    pub values: Rc<RefCell<HashMap<u64, (Value, bool)>>>,
    pub functions: Rc<RefCell<HashMap<u64, Value>>>,
    pub constructor: Option<Box<Value>>,
}

impl PartialOrd for TClass {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

impl TClass {
    pub fn new(
        name: Rc<str>,
        values: Rc<RefCell<HashMap<u64, (Value, bool)>>>,
        functions: Rc<RefCell<HashMap<u64, Value>>>,
        constructor: Option<Box<Value>>,
    ) -> Self {
        Self {
            name,
            values,
            functions,
            constructor,
        }
    }
}

impl Debug for TClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Class:{}", self.name)).unwrap();
        Ok(())
    }
}

// MEMBER ACCESS
impl IMemberAccessible for TClass {
    fn get_member(&self, _vm: &mut VM, member: &Value) -> Value {
        if let Value::String(member) = member {
            if let Some((v, _is_const)) = self.values.borrow().get(&hash_u64!(&member.0)) {
                return v.clone();
            }
            if let Some(v) = self.functions.borrow().get(&hash_u64!(&member.0)) {
                return v.clone();
            }
        }

        panic!("Cannot get member `{}` on {self:?}", member.to_string(true));
    }

    fn get_member_id(&self, vm: &mut VM, member: &u64) -> Value {
        if let Some((v, _is_const)) = self.values.borrow().get(member) {
            return v.clone();
        }
        if let Some(v) = self.functions.borrow().get(member) {
            return v.clone();
        }

        panic!(
            "Cannot get member id `{}` on {self:?}",
            vm.lookup_intern(*member)
        );
    }

    fn set_member(&mut self, member: &Value, value: Value) {
        if let Value::String(member) = member {
            if let Some((v, is_const)) = self.values.borrow_mut().get_mut(&hash_u64!(&member.0)) {
                if *is_const {
                    panic!("Cannot set constant member `{}` on {self:?}", member.0);
                }
                *v = value;
                return;
            }
        }

        panic!("Cannot set member `{}` on {self:?}", member.to_string(true));
    }

    fn set_member_id(&mut self, vm: &mut VM, member: &u64, value: Value) {
        if let Some((v, is_const)) = self.values.borrow_mut().get_mut(member) {
            if *is_const {
                panic!(
                    "Cannot set constant member id `{}` on {self:?}",
                    vm.lookup_intern(*member)
                );
            }
            *v = value;
            return;
        }

        panic!(
            "Cannot set member id `{}` on {self:?}",
            vm.lookup_intern(*member)
        );
    }
}
