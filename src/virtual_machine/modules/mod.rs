use crate::virtual_machine::{
    inst::Inst, traits::member_accessible::IMemberAccessible, value::Value, vm::VM,
};
use bincode::{Decode, Encode};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt::Debug,
    rc::Rc,
};

#[derive(Encode, Decode, Clone, PartialEq)]
pub struct Module {
    pub name: Rc<str>,
    pub path: Rc<str>,

    pub globals: Rc<RefCell<HashMap<u64, (Value, bool)>>>,
    pub exports: HashSet<u64>,

    pub instructions: Rc<RefCell<Vec<Inst>>>,
    pub cached: bool,
}

impl Debug for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Module(\"{}\")", self.name)).unwrap();

        Ok(())
    }
}

impl PartialOrd for Module {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

impl IMemberAccessible for Module {
    fn get_member(&self, _vm: &mut VM, member: &Value) -> Value {
        if let Value::String(member) = member {
            let id = &hash_u64!(&member.0);
            if self.exports.contains(id)
                && let Some((v, _)) = self.globals.borrow().get(id)
            {
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
        if self.exports.contains(member)
            && let Some((v, _)) = self.globals.borrow().get(member)
        {
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
            let id = &hash_u64!(&member.0);
            if self.exports.contains(id)
                && let Some((v, is_const)) = self.globals.borrow_mut().get_mut(id)
            {
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
        if self.exports.contains(member)
            && let Some((v, is_const)) = self.globals.borrow_mut().get_mut(member)
        {
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
