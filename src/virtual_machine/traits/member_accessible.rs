use crate::virtual_machine::{value::Value, vm::VM};
use std::fmt::Debug;

pub trait IMemberAccessible: Debug {
    fn get_member(&self, _vm: &mut VM, member: &Value) -> Value {
        panic!("Cannot get member `{}` on {self:?}", member.to_string(true));
    }

    #[allow(unused)]
    fn set_member(&mut self, member: &Value, _value: Value) {
        panic!("Cannot set member `{}` on {self:?}", member.to_string(true));
    }

    fn get_member_id(&self, vm: &mut VM, member: &u64) -> Value {
        self.get_member(vm, &Value::string(member.to_string()))
    }

    fn set_member_id(&mut self, member: &u64, value: Value) {
        self.set_member(&Value::string(member.to_string()), value)
    }
}
