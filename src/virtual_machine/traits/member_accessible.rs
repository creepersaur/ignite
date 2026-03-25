use crate::virtual_machine::{value::Value, vm::VM};
use std::fmt::Debug;

pub trait IMemberAccessible: Debug {
    fn get_member(&self, _vm: &mut VM, member: &Value) -> Value {
        panic!("Cannot get member `{}` on {self:?}", member.to_string(true));
    }

    #[allow(unused)]
    fn set_member(&self, member: &Value, _value: Value) {
        panic!("Cannot set member `{}` on {self:?}", member.to_string(true));
    }
}
