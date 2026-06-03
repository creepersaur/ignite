use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use crate::virtual_machine::{
    traits::member_accessible::IMemberAccessible, types::classes::class::TClass, value::Value,
    vm::VM,
};
use bincode::{Decode, Encode};

#[derive(Encode, Decode, Clone, PartialEq)]
pub struct TClassObject {
    pub base: Rc<RefCell<TClass>>,
    pub values: Rc<RefCell<HashMap<Rc<str>, (Value, bool)>>>,
}

impl PartialOrd for TClassObject {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

impl TClassObject {
    pub fn new(base: Rc<RefCell<TClass>>) -> Self {
        let values = base.borrow().values.clone();
        Self { base, values }
    }
}

impl Debug for TClassObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Object:{}", self.base.borrow().name))
            .unwrap();
        Ok(())
    }
}

// MEMBER ACCESS
impl IMemberAccessible for TClassObject {
    fn get_member(&self, _vm: &mut VM, member: &Value) -> Value {
        if let Value::String(member) = member {
            if let Some((v, _is_const)) = self.values.borrow().get(&*member.0) {
                return v.clone();
            }

            if let Some(v) = self
                .base
                .borrow()
                .functions
                .borrow_mut()
                .get_mut(&*member.0)
            {
                if let Value::Function(f) = v {
                    f.target = Some(Box::new(Value::ClassObject(self.clone())))
                }

                return v.clone();
            }
        }

        panic!("Cannot get member `{}` on {self:?}", member.to_string(true));
    }

    fn set_member(&mut self, member: &Value, value: Value) {
        if let Value::String(member) = member {
            if let Some((v, is_const)) = self.values.borrow_mut().get_mut(&*member.0) {
                if *is_const {
                    panic!("Cannot set constant member `{}` on {self:?}", member.0);
                }
                *v = value;
                return;
            } else {
                self.values
                    .borrow_mut()
                    .insert(member.0.clone(), (value, false));
                return;
            }
        }

        panic!("Cannot set member `{}` on {self:?}", member.to_string(true));
    }
}
