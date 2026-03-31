use crate::{
    hash_u64,
    rc,
    virtual_machine::{
        libs::lib::Library,
        types::{list::TList, string::TString},
        value::Value,
        vm::VM,
    },
};
use std::cell::RefCell;

pub const STRING_FUNCTIONS: [&str; 9] = [
    "len", "concat", "copy", "count", "reverse",
    "rep", "chars", "bytes", "split",
];

pub struct StringLib;

impl StringLib {
    fn len(vm: &mut VM) -> Value {
        let string = vm.pop();

        if let Value::String(inner) = string {
            Value::Number(inner.0.len() as f64)
        } else {
            panic!("Can only use string.len on strings");
        }
    }

    fn concat(vm: &mut VM) -> Value {
        let (other, string) = vm.pop_two();

        if let Value::String(inner) = string {
            return Value::String(TString::new(format!(
                "{}{}",
                inner.0,
                other.to_string(false)
            )));
        } else {
            panic!("Can only use string.concat on strings");
        }
    }

    fn copy(vm: &mut VM) -> Value {
        let string = vm.pop();

        if let Value::String(inner) = string {
            Value::String(TString(std::rc::Rc::from(&*inner.0)))
        } else {
            panic!("Can only use string.copy on strings");
        }
    }

    fn count(vm: &mut VM) -> Value {
        let (item, string) = vm.pop_two();

        if let Value::String(inner) = string {
            let count = inner.0.matches(&*item.to_string(false)).count();
            Value::Number(count as f64)
        } else {
            panic!("Can only use string.count on strings");
        }
    }

    fn reverse(vm: &mut VM) -> Value {
        let string = vm.pop();

        if let Value::String(inner) = string {
            Value::String(TString::new(inner.0.chars().rev().collect::<String>()))
        } else {
            panic!("Can only use string.reverse on strings");
        }
    }

    fn rep(vm: &mut VM) -> Value {
        let (value, string) = vm.pop_two();

        if let Value::String(inner) = string {
            if let Value::Number(n) = value {
                return Value::String(TString::new(inner.0.repeat(n as usize)));
            } else {
                panic!("Can only string.repeat with a number")
            }
        } else {
            panic!("Can only use string.repeat on strings");
        }
    }

    fn bytes(vm: &mut VM) -> Value {
        let string = vm.pop();

        if let Value::String(inner) = string {
            return Value::List(TList::new(rc!(RefCell::new(
                inner
                    .0
                    .bytes()
                    .map(|x| Value::Number(x as f64))
                    .collect::<Vec<_>>()
            ))));
        } else {
            panic!("Can only use string.bytes on strings");
        }
    }

    fn chars(vm: &mut VM) -> Value {
        let string = vm.pop();

        if let Value::String(inner) = string {
            return Value::List(TList::new(rc!(RefCell::new(
                inner.0.chars().map(|x| Value::Char(x)).collect::<Vec<_>>()
            ))));
        } else {
            panic!("Can only use string.chars on strings");
        }
    }

    fn split(vm: &mut VM) -> Value {
        let string = vm.pop();
        let new_value = vm.pop_or_nil();

        if let Value::String(inner) = string {
            if let Value::String(value) = new_value {
                Value::List(TList::new(rc!(RefCell::new(
                    inner
                        .0
                        .split(&*value.0)
                        .map(|x: &str| Value::String(TString::new(x.to_string())))
                        .collect::<Vec<_>>()
                ))))
            } else if let Value::Char(c) = new_value {
                Value::List(TList::new(rc!(RefCell::new(
                    inner
                        .0
                        .split(c)
                        .map(|x: &str| Value::String(TString::new(x.to_string())))
                        .collect::<Vec<_>>()
                ))))
            } else if Value::NIL == new_value {
                Value::List(TList::new(rc!(RefCell::new(
                    inner
                        .0
                        .split(" ")
                        .map(|x: &str| Value::String(TString::new(x.to_string())))
                        .collect::<Vec<_>>()
                ))))
            } else {
                panic!("Can only split() a string value with a string separator")
            }
        } else {
            panic!("Can only use string.split on strings")
        }
    }
}

// LIBRARY
impl Library for StringLib {
    fn get_name(&self) -> &str {
        "string"
    }

    fn get_function(&self, name: u64) -> Box<dyn Fn(&mut VM) -> Value> {
        match name {
            x if x == hash_u64!("len") => return Box::new(Self::len),
            x if x == hash_u64!("concat") => return Box::new(Self::concat),
            x if x == hash_u64!("copy") => return Box::new(Self::copy),
            x if x == hash_u64!("count") => return Box::new(Self::count),
            x if x == hash_u64!("reverse") => return Box::new(Self::reverse),
            x if x == hash_u64!("rep") => return Box::new(Self::rep),
            x if x == hash_u64!("bytes") => return Box::new(Self::bytes),
            x if x == hash_u64!("chars") => return Box::new(Self::chars),
            x if x == hash_u64!("split") => return Box::new(Self::split),

            _ => panic!("Unknown function `{name}` on lib {}", self.get_name()),
        }
    }
}
