use crate::virtual_machine::{traits::member_accessible::IMemberAccessible, value::Value, vm::VM};
use bincode::{Decode, Encode};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    rc::Rc,
};

#[derive(Debug, Clone, Encode, Decode)]
pub struct TNamespace {
    pub name: Rc<str>,
    pub locked: bool,
    pub env: HashMap<u64, (Value, bool)>,
}

impl TNamespace {
    pub fn new(name: &str, locked: bool) -> Self {
        Self {
            name: Rc::from(name),
            locked,
            env: HashMap::new(),
        }
    }

    #[allow(unused)]
    pub fn set(&mut self, name: &str, value: Value) {
        self.env.insert(hash_u64!(name), (value, false));
    }

    pub fn set_const(&mut self, name: &str, value: Value) {
        self.env.insert(hash_u64!(name), (value, true));
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
            if let Some((value, _)) = self.env.get(&hash_u64!(&t.0)) {
                return value.clone();
            } else {
                panic!(
                    "Tried to get unknown member `{}` on namespace:{}",
                    member.to_string(true),
                    self.name
                )
            }
        } else {
            panic!("Can only get string members on a namespace.")
        }
    }

    fn get_member_id(&self, vm: &mut VM, member: &u64) -> Value {
        if let Some((value, _)) = self.env.get(&member) {
            return value.clone();
        } else {
            panic!(
                "Tried to get unknown member id `{}` on namespace:{}",
                vm.lookup_intern(*member),
                self.name
            )
        }
    }

    fn set_member(&mut self, member: &Value, value: Value) {
        if self.locked {
            panic!("Cannot set a member of locked namespace `{}`", self.name);
        }

        if let Value::String(t) = member {
            if let Some((_, is_const)) = self.env.get(&hash_u64!(&t.0)) {
                if !*is_const {
                    self.env.insert(hash_u64!(&t.0), (value, false));
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

    fn set_member_id(&mut self, vm: &mut VM, member: &u64, value: Value) {
        if self.locked {
            panic!("Cannot set a member of locked namespace `{}`", self.name);
        }

        if let Some((_, is_const)) = self.env.get(member) {
            if !*is_const {
                self.env.insert(*member, (value, false));
            } else {
                panic!(
                    "Cannot set a constant member id `{}` of namespace:{}.",
                    vm.lookup_intern(*member),
                    self.name
                )
            }
        }
    }
}

// NAMESPACE LIB FUNCTION

#[macro_export]
macro_rules! namespace_lib_function {
    ($namespace:expr, $func:expr) => {
        $namespace.env.insert(
            hash_u64!($func),
            (lib_function!($namespace.name.clone(), $func), true),
        );
    };
}
