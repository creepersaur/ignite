use std::{collections::HashMap, fmt::Debug, rc::Rc};

use crate::virtual_machine::{traits::member_accessible::IMemberAccessible, value::Value, vm::VM};
use bincode::{Decode, Encode};

#[derive(Encode, Decode, Clone, PartialEq)]
pub struct TEnum {
	pub name: String,
    pub values: Rc<HashMap<Value, Value>>,
}

impl PartialOrd for TEnum {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

impl TEnum {
    pub fn new(name: String, values: Rc<HashMap<Value, Value>>) -> Self {
        Self { name, values }
    }
}

impl Debug for TEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Enum").unwrap();
        Ok(())
    }
}

// MEMBER ACCESS
impl IMemberAccessible for TEnum {
    fn get_member(&self, _vm: &mut VM, member: &Value) -> Value {
        if let Some(x) = self.values.get(member) {
            return x.clone();
        }

        panic!("Cannot get member `{}` on {self:?}", member.to_string(true));
    }
}
