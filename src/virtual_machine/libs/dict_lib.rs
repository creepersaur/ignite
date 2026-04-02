// return Value::Number(self.values.borrow().len() as f64)

use std::{cell::RefCell, collections::HashMap};

use crate::{
    get_args, hash_u64, rc, virtual_machine::{
        libs::lib::Library,
        types::{dict::TDict, list::TList},
        value::Value,
        vm::VM,
    }
};

pub const DICT_FUNCTIONS: [&str; 11] = [
    "len", "items", "keys", "values", "get", "insert", "remove", "clear", "append", "concat",
    "count",
];

pub struct DictLib;

impl DictLib {
    fn len(_vm: &mut VM, args: Vec<Value>) -> Value {
        let dict = get_args!(args);

        if let Value::Dict(inner) = dict {
            Value::Number(inner.values.borrow().len() as f64)
        } else {
            panic!("Can only use dict.len on Dicts");
        }
    }

    fn items(_vm: &mut VM, args: Vec<Value>) -> Value {
        let dict = get_args!(args);

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

    fn keys(_vm: &mut VM, args: Vec<Value>) -> Value {
        let dict = get_args!(args);

        if let Value::Dict(inner) = dict {
            let new_values = inner.values.borrow();

            return Value::List(TList::new(rc!(RefCell::new(
                new_values.keys().map(|x| x.clone()).collect::<Vec<_>>()
            ))));
        } else {
            panic!("Can only use dict.keys on Dicts")
        }
    }

    fn values(_vm: &mut VM, args: Vec<Value>) -> Value {
        let dict = get_args!(args);

        if let Value::Dict(inner) = dict {
            let new_values = inner.values.borrow();

            return Value::List(TList::new(rc!(RefCell::new(
                new_values.values().map(|x| x.clone()).collect::<Vec<_>>()
            ))));
        } else {
            panic!("Can only use dict.values on Dicts")
        }
    }

    fn get(_vm: &mut VM, args: Vec<Value>) -> Value {
        let dict = get_args!(args);
        let key = &args[1];

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

    fn insert(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [dict, new_value, key] = get_args!(args, 3);

        if let Value::Dict(x) = dict {
            let values = x.values;
            values.borrow_mut().insert(key, new_value);
        } else {
            panic!("Can only use dict.insert on dicts");
        }

        Value::NIL
    }

    fn remove(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [dict, key] = get_args!(args, 2);

        if let Value::Dict(inner) = dict {
            return inner.values.borrow_mut().remove(&key).unwrap_or(Value::NIL);
        } else {
            panic!("Can only use dict.remove on Dicts");
        }
    }

    fn clear(_vm: &mut VM, args: Vec<Value>) -> Value {
        let dict = get_args!(args);

        if let Value::Dict(inner) = dict {
            inner.values.borrow_mut().clear();
            return Value::NIL;
        } else {
            panic!("Can only use dict.clear on Dicts");
        }
    }

    fn map(vm: &mut VM, args: Vec<Value>) -> Value {
        let dict = get_args!(args);
        let func = &args[1];
        let mut new_map = HashMap::new();

        if let Value::Dict(inner) = dict {
            if let Value::Function(f) = func {
                for (key, value) in inner.values.borrow().iter() {
                    vm.stack.push(value.clone());
                    vm.stack.push(key.clone());
                    vm.call_function(f.clone(), 2);
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

    fn append(_vm: &mut VM, args: Vec<Value>) -> Value {
        let dict = get_args!(args);
        let other = &args[1];

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

    fn concat(_vm: &mut VM, args: Vec<Value>) -> Value {
        let dict = get_args!(args);
        let other = &args[1];

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

    fn copy(_vm: &mut VM, args: Vec<Value>) -> Value {
        let dict = get_args!(args);

        if let Value::Dict(inner) = dict {
            Value::Dict(inner.clone())
        } else {
            panic!("Can only use dict.copy on Dicts");
        }
    }

    fn count(_vm: &mut VM, args: Vec<Value>) -> Value {
        let dict = get_args!(args);
        let item = &args[1];

        if let Value::Dict(inner) = dict {
            let count = inner
                .values
                .borrow()
                .iter()
                .filter(|(_, x)| *x == item)
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

    fn get_function(&self, name: u64) -> Box<dyn Fn(&mut VM, Vec<Value>) -> Value> {
        match name {
            x if x == hash_u64!("len") => return Box::new(Self::len),
            x if x == hash_u64!("items") => return Box::new(Self::items),
            x if x == hash_u64!("keys") => return Box::new(Self::keys),
            x if x == hash_u64!("values") => return Box::new(Self::values),
            x if x == hash_u64!("get") => return Box::new(Self::get),
            x if x == hash_u64!("insert") => return Box::new(Self::insert),
            x if x == hash_u64!("remove") => return Box::new(Self::remove),
            x if x == hash_u64!("map") => return Box::new(Self::map),
            x if x == hash_u64!("clear") => return Box::new(Self::clear),
            x if x == hash_u64!("append") => return Box::new(Self::append),
            x if x == hash_u64!("concat") => return Box::new(Self::concat),
            x if x == hash_u64!("copy") => return Box::new(Self::copy),
            x if x == hash_u64!("count") => return Box::new(Self::count),

            _ => panic!("Unknown function `{name}` on lib {}", self.get_name()),
        }
    }
}
