use crate::{
    rc,
    virtual_machine::{traits::member_accessible::IMemberAccessible, value::Value, vm::VM},
};
use std::{collections::HashMap, rc::Rc};

#[derive(Debug, Clone)]
pub struct Namespace {
    pub name: String,
    pub locked: bool,
    pub env: HashMap<Rc<String>, (Value, bool)>,
}

impl IMemberAccessible for Namespace {
    fn get_member(&self, _vm: &mut VM, member: &Value) -> Value {
        if let Value::String(t) = member {
            if let Some((value, _)) = self.env.get(&*t.0.borrow()) {
                return value.clone();
            } else {
                panic!(
                    "Tried to get unknown member {} on namespace:{}",
                    member.to_string(true),
                    self.name
                )
            }
        } else {
            panic!("Can only get string members on a namespace.")
        }
    }

    fn set_member(&self, member: &Value, value: Value) {
        if self.locked {
            panic!("Cannot set a member of locked namespace `{}`", self.name);
        }

        if let Value::String(t) = member {
            if let Some((_, is_const)) = self.env.get(&t.0.borrow()) {
                if !*is_const {
                    self.env.insert(rc!(t.0.borrow().clone()), (value, false));
                } else {
                    panic!(
                        "Cannot set a constant member `{}` of namespace:{}.",
                        member.to_string(false),
                        self.name
                    )
                }
            }
        } else {
            panic!("Can only set string members on a namespace.")
        }
    }
}
