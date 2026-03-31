use crate::virtual_machine::{traits::member_accessible::IMemberAccessible, value::Value, vm::VM};
use bincode::{Decode, Encode};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    rc::Rc,
};

#[derive(Debug, Clone, Encode, Decode)]
pub struct TNamespace {
    pub name: String,
    pub locked: bool,
    pub env: HashMap<Rc<str>, (Value, bool)>,
}

impl TNamespace {
    pub fn new(name: &str, locked: bool) -> Self {
        Self {
            name: name.to_string(),
            locked,
            env: HashMap::new(),
        }
    }

    #[allow(unused)]
    pub fn set(&mut self, name: &str, value: Value) {
        self.env.insert(Rc::from(name), (value, false));
    }

    pub fn set_const(&mut self, name: &str, value: Value) {
        self.env.insert(Rc::from(name), (value, true));
    }
}

impl PartialEq for TNamespace {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl PartialOrd for TNamespace {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Hash for TNamespace {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl IMemberAccessible for TNamespace {
    fn get_member(&self, _vm: &mut VM, member: &Value) -> Value {
        if let Value::String(t) = member {
            if let Some((value, _)) = self.env.get(&*t.0) {
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

    fn set_member(&mut self, member: &Value, value: Value) {
        if self.locked {
            panic!("Cannot set a member of locked namespace `{}`", self.name);
        }

        if let Value::String(t) = member {
            if let Some((_, is_const)) = self.env.get(&*t.0) {
                if !*is_const {
                    self.env.insert(t.0.clone(), (value, false));
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

// NAMESPACE LIB FUNCTION

#[macro_export]
macro_rules! namespace_lib_function {
    ($namespace:expr, $lib:expr, $func:expr, $args:expr, $return_type:expr) => {
        $namespace.env.insert(
            std::rc::Rc::from($func),
            (lib_function!($lib, $func, $args, $return_type), true),
        );
    };
}
