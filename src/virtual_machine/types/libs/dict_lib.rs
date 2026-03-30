// return Value::Number(self.values.borrow().len() as f64)

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    rc,
    virtual_machine::{
        libs::lib::Library,
        types::{dict::TDict, list::TList},
        value::Value,
        vm::VM,
    },
};

pub const DICT_FUNCTIONS: [&str; 11] = [
    "len", "items", "keys", "values", "get", "insert", "remove", "clear", "append", "concat",
    "count",
];

pub struct DictLib;

impl DictLib {
    fn len(vm: &mut VM) -> Value {
        let dict = vm.pop();

        if let Value::Dict(inner) = dict {
            Value::Number(inner.values.borrow().len() as f64)
        } else {
            panic!("Can only use dict.len on Dicts");
        }
    }

    fn items(vm: &mut VM) -> Value {
        let dict = vm.pop();

        if let Value::Dict(inner) = dict {
            let new_values = inner.values.borrow();

            return Value::List(TList::new(rc!(RefCell::new(
                new_values
                    .iter()
                    .map(
                        |(k, v)| Value::Tuple(TList::new_tuple(rc!(RefCell::new(vec![
                            k.clone(),
                            v.clone()
                        ]))))
                    )
                    .collect::<Vec<_>>()
            ))));
        } else {
            panic!("Can only use dict.items on Dicts")
        }
    }

    fn keys(vm: &mut VM) -> Value {
        let dict = vm.pop();

        if let Value::Dict(inner) = dict {
            let new_values = inner.values.borrow();

            return Value::List(TList::new(rc!(RefCell::new(
                new_values.keys().map(|x| x.clone()).collect::<Vec<_>>()
            ))));
        } else {
            panic!("Can only use dict.keys on Dicts")
        }
    }

    fn values(vm: &mut VM) -> Value {
        let dict = vm.pop();

        if let Value::Dict(inner) = dict {
            let new_values = inner.values.borrow();

            return Value::List(TList::new(rc!(RefCell::new(
                new_values.values().map(|x| x.clone()).collect::<Vec<_>>()
            ))));
        } else {
            panic!("Can only use dict.values on Dicts")
        }
    }

    fn get(vm: &mut VM) -> Value {
        let dict = vm.pop();
        let key = vm.pop();

        if let Value::Dict(inner) = dict {
            return inner
                .values
                .borrow()
                .get(&key)
                .unwrap_or(&Value::NIL)
                .clone();
        } else {
            panic!("Can only use dict.len on Dicts");
        }
    }

    fn insert(vm: &mut VM) -> Value {
        let dict = vm.pop();
        let new_value = vm.pop();
        let key = vm.pop();

        if let Value::Dict(x) = dict {
            let values = x.values;
            values.borrow_mut().insert(key, new_value);
        } else {
            panic!("Can only use dict.insert on dicts");
        }

        Value::NIL
    }

    fn remove(vm: &mut VM) -> Value {
        let (key, dict) = vm.pop_two();

        if let Value::Dict(inner) = dict {
            return inner.values.borrow_mut().remove(&key).unwrap_or(Value::NIL);
        } else {
            panic!("Can only use dict.remove on Dicts");
        }
    }

    fn clear(vm: &mut VM) -> Value {
        let list = vm.pop();

        if let Value::Dict(inner) = list {
            inner.values.borrow_mut().clear();
            return Value::NIL;
        } else {
            panic!("Can only use dict.clear on Dicts");
        }
    }

    fn map(vm: &mut VM) -> Value {
        let (func, dict) = vm.pop_two();
        let mut new_map = HashMap::new();

        if let Value::Dict(inner) = dict {
            if let Value::Function(f) = func {
                for (key, value) in inner.values.borrow().iter() {
                    vm.stack.push(value.clone());
                    vm.stack.push(key.clone());
                    vm.call_function(f.clone());
                    vm.run(false, true);
                    let new_value = vm.pop();
                    new_map.insert(key.clone(), new_value);
                }

                return Value::Dict(TDict::new(rc!(RefCell::new(new_map))));
            } else {
                panic!("Expected function in dict.map");
            }
        } else {
            panic!("Can only use dict.map on Dicts");
        }
    }

    fn append(vm: &mut VM) -> Value {
        let (other, dict) = vm.pop_two();

        if let Value::Dict(inner) = dict {
            if let Value::Dict(other_inner) = other {
                inner
                    .values
                    .borrow_mut()
                    .extend(other_inner.values.borrow().clone());
            } else {
                panic!("Can only `append` another Dict on a Dict")
            }
        } else {
            panic!("Can only use dict.append on Dict");
        }

        Value::NIL
    }

    fn concat(vm: &mut VM) -> Value {
        let (other, dict) = vm.pop_two();

        if let Value::Dict(inner) = dict {
            if let Value::Dict(other_inner) = other {
                let new_values = inner
                    .values
                    .borrow()
                    .clone()
                    .into_iter()
                    .chain(other_inner.values.borrow().clone())
                    .collect::<HashMap<_, _>>();

                return Value::Dict(TDict::new(rc!(RefCell::new(new_values))));
            } else {
                panic!("Can only `concat` another with a Dict")
            }
        } else {
            panic!("Can only use dict.concat on Dict");
        }
    }

    fn copy(vm: &mut VM) -> Value {
        let dict = vm.pop();

        if let Value::Dict(inner) = dict {
            Value::Dict(inner.clone())
        } else {
            panic!("Can only use dict.copy on Dicts");
        }
    }

    fn count(vm: &mut VM) -> Value {
        let (item, dict) = vm.pop_two();

        if let Value::Dict(inner) = dict {
            let count = inner
                .values
                .borrow()
                .iter()
                .filter(|(_, x)| x == &&item)
                .count();
            Value::Number(count as f64)
        } else {
            panic!("Can only use dict.count on Dicts");
        }
    }
}

// LIBRARY
impl Library for DictLib {
    fn get_name(&self) -> &str {
        "dict"
    }

    fn get_function(&self, name: Rc<String>) -> Box<dyn Fn(&mut VM) -> Value> {
        match name.as_str() {
            "len" => return Box::new(Self::len),
            "items" => return Box::new(Self::items),
            "keys" => return Box::new(Self::keys),
            "values" => return Box::new(Self::values),
            "get" => return Box::new(Self::get),
            "insert" => return Box::new(Self::insert),
            "remove" => return Box::new(Self::remove),
            "map" => return Box::new(Self::map),
            "clear" => return Box::new(Self::clear),
            "append" => return Box::new(Self::append),
            "concat" => return Box::new(Self::concat),
            "copy" => return Box::new(Self::copy),
            "count" => return Box::new(Self::count),

            _ => panic!("Unknown function `{name}` on lib {}", self.get_name()),
        }
    }
}
