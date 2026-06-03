use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use crate::virtual_machine::{
    traits::member_accessible::IMemberAccessible, types::structdef::TStructDef, value::Value,
    vm::VM,
};
use bincode::{Decode, Encode};

#[derive(Encode, Decode, Clone, PartialEq)]
pub struct TStruct {
    pub base: Rc<TStructDef>,
    pub values: Rc<RefCell<HashMap<Rc<str>, Value>>>,
}

impl PartialOrd for TStruct {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

impl TStruct {
    pub fn new(base: Rc<TStructDef>, values: Rc<RefCell<HashMap<Rc<str>, Value>>>) -> Self {
        Self { base, values }
    }
}

impl Debug for TStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Struct:{}", self.base.name)).unwrap();
        Ok(())
    }
}

// MEMBER ACCESS
impl IMemberAccessible for TStruct {
    fn get_member(&self, _vm: &mut VM, member: &Value) -> Value {
        if let Value::String(member) = member {
            if let Some(v) = self.values.borrow().get(&*member.0) {
                return v.clone();
            }
        }

        panic!("Cannot get member `{}` on {self:?}", member.to_string(true));
    }

    fn set_member(&mut self, member: &Value, value: Value) {
        if let Value::String(member) = member {
            if let Some(v_type) = self.base.fields.get(&*member.0) {
                if !value.type_matches(v_type) {
                    panic!(
                        "Field '{}' expects type `{v_type}`, got `{}`. (Struct '{}')",
                        &*member.0,
                        value.get_type(),
                        self.base.name
                    )
                }
            } else {
                panic!(
                    "Tried setting unknown field on struct of base {}",
                    self.base.name
                );
            }
            if let Some(v) = self.values.borrow_mut().get_mut(&*member.0) {
                *v = value;
                return;
            }
        }

        panic!("Cannot set member `{}` on {self:?}", member.to_string(true));
    }
}
