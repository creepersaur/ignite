use std::{fmt::Debug, rc::Rc};

use crate::{
    misc::to_index::to_index,
    virtual_machine::{
        libs::types::string_lib::STRING_FUNCTIONS, traits::member_accessible::IMemberAccessible,
        types::function::TFunction, value::Value, vm::VM,
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
                return lib_function!(self, "string", member.0.clone(), Value::String);
            }
        }

        if let Value::Range {
            start,
            end,
            step,
            inclusive,
        } = member
        {
            let chars: Vec<char> = self.0.chars().collect();
            let len = chars.len();

            let start_i = if let Value::Number(n) = start.as_ref() {
                to_index(*n, len)
            } else {
                0
            };

            let end_i = if let Value::Number(n) = end.as_ref() {
                let i = to_index(*n, len);
                if *inclusive { i + 1 } else { i }
            } else {
                len
            };

            let step_by = if let Value::Number(n) = step.as_ref() {
                (*n as usize).max(1)
            } else {
                1
            };

            let slice: String = chars[start_i..end_i].iter().step_by(step_by).collect();

            return Value::String(TString::new(slice));
        }

        panic!("Cannot get member `{}` on {self:?}", member.to_string(true));
    }

    fn set_member(&mut self, member: &Value, _value: Value) {
        panic!("Cannot set member `{}` on {self:?}", member.to_string(true));
    }
}
