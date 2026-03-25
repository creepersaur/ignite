use std::{cell::RefCell, rc::Rc};

use crate::{
    misc::to_index::to_index,
    rc,
    virtual_machine::{libs::lib::Library, types::string::TString, value::Value, vm::VM},
};

pub struct StringLib;

impl StringLib {
    fn len(vm: &mut VM) -> Value {
        let string = vm.pop();

        if let Value::String(inner) = string {
            Value::Number(inner.0.borrow().len() as f32)
        } else {
            panic!("Can only use string.len on strings");
        }
    }

    fn push(vm: &mut VM) -> Value {
        let string = vm.pop();
        let new_value = vm.pop();

        if let Value::String(inner) = string {
            if let Value::String(value) = new_value {
                inner.0.borrow_mut().push_str(&value.0.borrow());
            } else {
                panic!("Can only push() a string value onto a string")
            }
        } else {
            panic!("Can only use string.push on strings")
        }

        Value::NIL
    }

    fn insert(vm: &mut VM) -> Value {
        let string = vm.pop();
        let new_value = vm.pop();
        let index = vm.pop();

        if let Value::String(inner) = string {
            if let Value::String(value) = new_value {
                if let Value::Number(idx) = index {
                    let target_index = to_index(idx, inner.0.borrow().len());
                    inner
                        .0
                        .borrow_mut()
                        .insert_str(target_index, &value.0.borrow());
                } else {
                    panic!("Expected number as index in string.insert");
                }
            } else {
                panic!("Can only insert() a string value onto a string")
            }
        } else {
            panic!("Can only use string.insert on strings");
        }

        Value::NIL
    }

    fn remove(vm: &mut VM) -> Value {
        let (index, string) = vm.pop_two();

        if let Value::String(inner) = string {
            if let Value::Number(idx) = index {
                let target_index = to_index(idx, inner.0.borrow().len());
                let ch = inner.0.borrow_mut().remove(target_index);
                return Value::String(TString(rc!(RefCell::new(ch.to_string()))));
            } else {
                panic!("Expected number as index in string.remove");
            }
        } else {
            panic!("Can only use string.remove on strings");
        }
    }

    fn pop(vm: &mut VM) -> Value {
        let string = vm.pop();

        if let Value::String(inner) = string {
            let popped = inner.0.borrow_mut().pop();
            if let Some(new_char) = popped {
                return Value::String(TString(rc!(RefCell::new(new_char.to_string()))));
            } else {
                return Value::NIL;
            }
        } else {
            panic!("Can only use string.pop on strings");
        }
    }

    fn clear(vm: &mut VM) -> Value {
        let string = vm.pop();

        if let Value::String(inner) = string {
            inner.0.borrow_mut().clear();
            return Value::NIL;
        } else {
            panic!("Can only use string.clear on strings");
        }
    }

    fn concat(vm: &mut VM) -> Value {
        let (other, string) = vm.pop_two();

        if let Value::String(inner) = string {
            return Value::String(TString(rc!(RefCell::new(format!(
                "{}{}",
                inner.0.borrow(),
                other.to_string(false)
            )))));
        } else {
            panic!("Can only use string.concat on strings");
        }
    }

    fn copy(vm: &mut VM) -> Value {
        let string = vm.pop();

        if let Value::String(inner) = string {
            Value::String(TString(rc!(RefCell::new(inner.0.borrow().clone()))))
        } else {
            panic!("Can only use string.copy on strings");
        }
    }

    fn count(vm: &mut VM) -> Value {
        let (item, string) = vm.pop_two();

        if let Value::String(inner) = string {
            let count = inner.0.borrow().matches(&*item.to_string(false)).count();
            Value::Number(count as f32)
        } else {
            panic!("Can only use string.count on strings");
        }
    }

    fn reverse(vm: &mut VM) -> Value {
        let string = vm.pop();

        if let Value::String(inner) = string {
            Value::String(TString(rc!(RefCell::new(
                inner.0.borrow().chars().rev().collect::<String>(),
            ))))
        } else {
            panic!("Can only use string.reverse on strings");
        }
    }

    fn fill(vm: &mut VM) -> Value {
        let (value, string) = vm.pop_two();

        if let Value::String(inner) = string {
            let len = inner.0.borrow().len();
            *inner.0.borrow_mut() = value.to_string(false).repeat(len);
        } else {
            panic!("Can only use string.fill on strings");
        }

        Value::NIL
    }

    fn rep(vm: &mut VM) -> Value {
        let (value, string) = vm.pop_two();

        if let Value::String(inner) = string {
            if let Value::Number(n) = value {
                return Value::String(TString(rc!(RefCell::new(
                    inner.0.borrow().repeat(n as usize),
                ))));
            } else {
                panic!("Can only string.repeat with a number")
            }
        } else {
            panic!("Can only use string.repeat on strings");
        }
    }

    fn push_n(vm: &mut VM) -> Value {
        let string = vm.pop();
        let value = vm.pop();
        let num = vm.pop();

        if let Value::String(inner) = string {
            if let Value::Number(n) = num {
                let repeated = value.to_string(false).repeat(n as usize);
                inner.0.borrow_mut().push_str(&repeated);
            } else {
                panic!("string.push_n requires a `number` of times as first argument")
            }
        } else {
            panic!("Can only use string.push_n on strings");
        }

        Value::NIL
    }
}

// LIBRARY
impl Library for StringLib {
    fn get_name(&self) -> &str {
        "string"
    }

    fn get_function(&self, name: Rc<String>) -> Box<dyn Fn(&mut VM) -> Value> {
        match name.as_str() {
            "len" => return Box::new(Self::len),
            "push" => return Box::new(Self::push),
            "insert" => return Box::new(Self::insert),
            "remove" => return Box::new(Self::remove),
            "pop" => return Box::new(Self::pop),
            "clear" => return Box::new(Self::clear),
            "concat" => return Box::new(Self::concat),
            "copy" => return Box::new(Self::copy),
            "count" => return Box::new(Self::count),
            "reverse" => return Box::new(Self::reverse),
            "fill" => return Box::new(Self::fill),
            "rep" => return Box::new(Self::rep),
            "push_n" => return Box::new(Self::push_n),

            _ => panic!("Unknown function `{name}` on lib {}", self.get_name()),
        }
    }
}
