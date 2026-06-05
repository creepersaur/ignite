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
        panic!("Cannot get member id `{}` on {self:?}", vm.lookup_intern(*member));
    }

    fn set_member_id(&mut self, vm: &mut VM, member: &u64, _value: Value) {
        panic!("Cannot set member id `{}` on {self:?}", vm.lookup_intern(*member));
    }
}
