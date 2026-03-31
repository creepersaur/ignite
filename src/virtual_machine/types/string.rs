use std::{fmt::Debug, rc::Rc};

use crate::{
    lib_function,
    misc::to_index::to_index,
    rc,
    virtual_machine::{
        traits::member_accessible::IMemberAccessible, types::function::TFunction,
        libs::string_lib::STRING_FUNCTIONS, value::Value, vm::VM,
    },
};
use bincode::{Decode, Encode};

#[derive(Encode, Decode, Clone, PartialEq, PartialOrd)]
pub struct TString(pub Rc<str>);

impl TString {
    pub fn new(s: String) -> Self {
        Self(Rc::from(s))
    }

	#[allow(unused)]
    pub fn from_str(s: &str) -> Self {
        Self(Rc::from(s))
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Debug for TString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("\"{}\"", self.0)).unwrap();
        Ok(())
    }
}

// MEMBER ACCESS
impl IMemberAccessible for TString {
    fn get_member(&self, _vm: &mut VM, member: &Value) -> Value {
        if let Value::Number(index) = member {
            let len = self.0.len();
            let target_index = to_index(*index, len);

            return Value::Char(self.0.chars().nth(target_index).unwrap());
        }

        if let Value::String(member) = member {
            if STRING_FUNCTIONS.contains(&&*member.0) {
                return lib_function!(self, "string", member.0.clone(), 1, Value::String);
            }
        }

        panic!("Cannot get member `{}` on {self:?}", member.to_string(true));
    }

    fn set_member(&mut self, member: &Value, _value: Value) {
        panic!("Cannot set member `{}` on {self:?}", member.to_string(true));
    }
}
