use crate::virtual_machine::{
    inst::Inst, traits::member_accessible::IMemberAccessible, value::Value, vm::VM,
};
use bincode::{Decode, Encode};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct Module {
    pub name: Rc<str>,
    pub path: Rc<str>,
    pub cached: bool,
    pub exports: HashMap<u64, (Value, bool)>,
    pub instructions: Rc<RefCell<Vec<Inst>>>,
}

impl PartialOrd for Module {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

impl IMemberAccessible for Module {
    fn get_member(&self, _vm: &mut VM, member: &Value) -> Value {
        if let Value::String(member) = member {
            if let Some((v, _)) = self.exports.get(&hash_u64!(&member.0)) {
                return v.clone();
            }
        }

        panic!(
            "Cannot get member `{}` on module(\"{}\")",
            member.to_string(true),
            self.name
        );
    }

    fn get_member_id(&self, vm: &mut VM, member: &u64) -> Value {
        if let Some((v, _)) = self.exports.get(member) {
            return v.clone();
        }

        panic!(
            "Cannot get member id `{}` on module(\"{}\")",
            vm.lookup_intern(*member),
            self.name
        );
    }

    fn set_member(&mut self, member: &Value, value: Value) {
        if let Value::String(member) = member {
            if let Some((v, is_const)) = self.exports.get_mut(&hash_u64!(&member.0)) {
                if *is_const {
                    panic!(
                        "Tried setting constant member `{}` on module(\"{}\")",
                        member.0, self.name
                    );
                }

                *v = value;
            } else {
                panic!(
                    "Tried setting unknown member `{}` on module(\"{}\")",
                    member.0, self.name
                );
            }
        }

        panic!(
            "Cannot set member `{}` on module(\"{}\")",
            member.to_string(true),
            self.name
        );
    }

    fn set_member_id(&mut self, vm: &mut VM, member: &u64, value: Value) {
        if let Some((v, is_const)) = self.exports.get_mut(member) {
            if *is_const {
                panic!(
                    "Tried setting constant member `{}` on module(\"{}\")",
                    vm.lookup_intern(*member),
                    self.name
                );
            }

            *v = value;
        }

        panic!(
            "Cannot set member id `{}` on module(\"{}\")",
            vm.lookup_intern(*member),
            self.name
        );
    }
}
