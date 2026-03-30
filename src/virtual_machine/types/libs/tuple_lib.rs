use std::{cell::RefCell, cmp::Ordering, rc::Rc};

use crate::{
    rc,
    virtual_machine::{libs::lib::Library, types::list::TList, value::Value, vm::VM},
};

pub const TUPLE_FUNCTIONS: [&str; 10] = [
    "len", "insert", "map", "concat", "copy", "count", "sort", "reverse", "rep", "to_list",
];

pub struct TupleLib;

impl TupleLib {
    fn to_list(vm: &mut VM) -> Value {
        let tuple = vm.pop();

        if let Value::Tuple(inner) = tuple {
            let mut new_inner = inner.clone();
            new_inner.is_tuple = true;
            Value::List(new_inner)
        } else {
            panic!("Can only use tuple.to_list on Tuples");
        }
    }

    fn len(vm: &mut VM) -> Value {
        let tuple = vm.pop();

        if let Value::Tuple(inner) = tuple {
            Value::Number(inner.values.borrow().len() as f64)
        } else {
            panic!("Can only use tuple.len on Tuples");
        }
    }

    fn map(vm: &mut VM) -> Value {
        let (func, tuple) = vm.pop_two();
        let mut new_array = vec![];

        if let Value::Tuple(inner) = tuple {
            if let Value::Function(f) = func {
                for i in inner.values.borrow().iter() {
                    vm.stack.push(i.clone());
                    vm.call_function(f.clone());
                    vm.run(false, true);
                    let new_value = vm.pop();
                    new_array.push(new_value);
                }

                return Value::Tuple(TList::new_tuple(rc!(RefCell::new(new_array))));
            } else {
                panic!("Expected function in tuple.map");
            }
        } else {
            panic!("Can only use tuple.map on Tuples");
        }
    }

    fn concat(vm: &mut VM) -> Value {
        let (other, tuple) = vm.pop_two();

        if let Value::Tuple(inner) = tuple {
            if let Value::Tuple(other_inner) = other {
                let new_values = [
                    inner.values.borrow().clone(),
                    other_inner.values.borrow().clone(),
                ]
                .concat();

                return Value::Tuple(TList::new_tuple(rc!(RefCell::new(new_values))));
            } else if let Value::List(other_inner) = other {
                let new_values = [
                    inner.values.borrow().clone(),
                    other_inner.values.borrow().clone(),
                ]
                .concat();

                return Value::Tuple(TList::new_tuple(rc!(RefCell::new(new_values))));
            } else {
                panic!("Can only `concat` another List/Tuple with a Tuple")
            }
        } else {
            panic!("Can only use tuple.concat on Tuples");
        }
    }

    fn copy(vm: &mut VM) -> Value {
        let tuple = vm.pop();

        if let Value::Tuple(inner) = tuple {
            Value::Tuple(inner.clone())
        } else {
            panic!("Can only use tuple.copy on Tuples");
        }
    }

    fn count(vm: &mut VM) -> Value {
        let (item, tuple) = vm.pop_two();

        if let Value::Tuple(inner) = tuple {
            let count = inner.values.borrow().iter().filter(|x| x == &&item).count();
            Value::Number(count as f64)
        } else {
            panic!("Can only use tuple.count on Tuples");
        }
    }

    fn sort(vm: &mut VM) -> Value {
        let tuple = vm.pop();

        if let Value::Tuple(inner) = tuple {
            inner
                .values
                .borrow_mut()
                .sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        } else {
            panic!("Can only use tuple.sort on Tuples");
        }

        Value::NIL
    }

    fn reverse(vm: &mut VM) -> Value {
        let tuple = vm.pop();

        if let Value::Tuple(inner) = tuple {
            inner.values.borrow_mut().reverse();
        } else {
            panic!("Can only use tuple.reverse on Tuples");
        }

        Value::NIL
    }

    fn rep(vm: &mut VM) -> Value {
        let (value, tuple) = vm.pop_two();

        if let Value::Tuple(inner) = tuple {
            if let Value::Number(n) = value {
                return Value::Tuple(TList::new_tuple(rc!(RefCell::new(
                    std::iter::repeat(&*inner.values.borrow())
                        .take(n as usize)
                        .flatten()
                        .cloned()
                        .collect()
                ))));
            } else {
                panic!("Can only tuple.repeat with a number")
            }
        } else {
            panic!("Can only use tuple.repeat on Tuples");
        }
    }
}

// LIBRARY
impl Library for TupleLib {
    fn get_name(&self) -> &str {
        "tuple"
    }

    fn get_function(&self, name: Rc<String>) -> Box<dyn Fn(&mut VM) -> Value> {
        match name.as_str() {
            "len" => return Box::new(Self::len),
            "map" => return Box::new(Self::map),
            "concat" => return Box::new(Self::concat),
            "copy" => return Box::new(Self::copy),
            "count" => return Box::new(Self::count),
            "sort" => return Box::new(Self::sort),
            "reverse" => return Box::new(Self::reverse),
            "rep" => return Box::new(Self::rep),
            "to_list" => return Box::new(Self::to_list),

            _ => panic!("Unknown function `{name}` on lib {}", self.get_name()),
        }
    }
}
