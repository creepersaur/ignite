use std::{collections::HashMap, fmt::Debug, rc::Rc};

use crate::virtual_machine::{traits::member_accessible::IMemberAccessible, value::Value, vm::VM};
use bincode::{Decode, Encode};

#[derive(Encode, Decode, Clone, PartialEq)]
pub struct TEnum {
    pub name: Rc<str>,
    pub values: Rc<HashMap<Rc<str>, Value>>,
}

impl PartialOrd for TEnum {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

impl TEnum {
    pub fn new(name: Rc<str>, values: Rc<HashMap<Rc<str>, Value>>) -> Self {
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
        if let Value::String(s) = member {
            if let Some(x) = self.values.get(&s.0) {
                return x.clone();
            }

            panic!("Cannot get member `{}` on {self:?}", member.to_string(true));
        }

        panic!(
            "Can only get string members on enum `{}`. Got {}",
            self.name,
            member.to_string(true)
        )
    }
}
