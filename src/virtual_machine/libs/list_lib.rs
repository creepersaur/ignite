use crate::{
    get_args, hash_u64, misc::to_index::to_index, rc, virtual_machine::{libs::lib::Library, types::list::TList, value::Value, vm::VM}
};
use std::{cell::RefCell, cmp::Ordering};

pub const LIST_FUNCTIONS: [&str; 16] = [
    "len", "push", "insert", "remove", "map", "pop", "clear", "append", "concat", "copy", "count",
    "sort", "reverse", "fill", "rep", "push_n",
];

pub struct ListLib;

impl ListLib {
    fn len(_vm: &mut VM, args: Vec<Value>) -> Value {
        let list = get_args!(args);

        if let Value::List(inner) = list {
            Value::Number(inner.values.borrow().len() as f64)
        } else {
            panic!("Can only use list.len on Lists");
        }
    }

    fn push(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [list, new_value] = get_args!(args, 2);

        if let Value::List(x) = list {
            x.values.borrow_mut().push(new_value);
        } else {
            panic!("Can only use list.push on Lists")
        }

        Value::NIL
    }

    fn insert(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [list, new_value, index] = get_args!(args, 3);

        if let Value::List(x) = list {
            x.values
                .borrow_mut()
                .insert(index.as_number() as usize, new_value);
        } else {
            panic!("Can only use list.insert on Lists");
        }

        Value::NIL
    }

    fn remove(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [index, list] = get_args!(args, 2);

        if let Value::List(inner) = list {
            if let Value::Number(idx) = index {
                let target_index = to_index(idx, inner.values.borrow().len());
                return inner.values.borrow_mut().remove(target_index);
            } else {
                panic!("Expected number as index in list.remove");
            }
        } else {
            panic!("Can only use list.remove on Lists");
        }
    }

    fn pop(_vm: &mut VM, args: Vec<Value>) -> Value {
        let list = get_args!(args);

        if let Value::List(inner) = list {
            inner.values.borrow_mut().pop().unwrap_or(Value::NIL)
        } else {
            panic!("Can only use list.pop on Lists");
        }
    }

    fn clear(_vm: &mut VM, args: Vec<Value>) -> Value {
        let list = get_args!(args);

        if let Value::List(inner) = list {
            inner.values.borrow_mut().clear();
        } else {
            panic!("Can only use list.clear on Lists");
        }

        Value::NIL
    }

    fn map(vm: &mut VM, args: Vec<Value>) -> Value {
        let [func, list] = get_args!(args, 2);
        let mut new_array = vec![];

        if let Value::List(inner) = list {
            if let Value::Function(f) = func {
                for i in inner.values.borrow().iter() {
                    vm.stack.push(i.clone());
                    vm.call_function(f.clone(), 1);
                    vm.run(false, true);
                    let new_value = vm.pop();
                    new_array.push(new_value);
                }

                return Value::List(TList::new(rc!(RefCell::new(new_array))));
            } else {
                panic!("Expected function in list.map");
            }
        } else {
            panic!("Can only use list.map on Lists");
        }
    }

    fn append(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [other, list] = get_args!(args, 2);

        if let Value::List(inner) = list {
            if let Value::List(other_inner) = other {
                inner
                    .values
                    .borrow_mut()
                    .extend(other_inner.values.borrow().clone());
            } else {
                panic!("Can only `append` another List/Tuple on a List")
            }
        } else {
            panic!("Can only use list.append on Lists");
        }

        Value::NIL
    }

    fn concat(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [other, list] = get_args!(args, 2);

        if let Value::List(inner) = list {
            if let Value::List(other_inner) = other {
                let new_values = [
                    inner.values.borrow().clone(),
                    other_inner.values.borrow().clone(),
                ]
                .concat();

                return Value::List(TList::new(rc!(RefCell::new(new_values))));
            } else {
                panic!("Can only `concat` another List/Tuple with a List")
            }
        } else {
            panic!("Can only use list.concat on Lists");
        }
    }

    fn copy(_vm: &mut VM, args: Vec<Value>) -> Value {
        let list = get_args!(args);

        if let Value::List(inner) = list {
            Value::List(inner.clone())
        } else {
            panic!("Can only use list.copy on Lists");
        }
    }

    fn count(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [item, list] = get_args!(args, 2);

        if let Value::List(inner) = list {
            let count = inner.values.borrow().iter().filter(|x| x == &&item).count();
            Value::Number(count as f64)
        } else {
            panic!("Can only use list.count on Lists");
        }
    }

    fn sort(_vm: &mut VM, args: Vec<Value>) -> Value {
        let list = get_args!(args);

        if let Value::List(inner) = list {
            inner
                .values
                .borrow_mut()
                .sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        } else {
            panic!("Can only use list.sort on Lists");
        }

        Value::NIL
    }

    fn reverse(_vm: &mut VM, args: Vec<Value>) -> Value {
        let list = get_args!(args);

        if let Value::List(inner) = list {
            inner.values.borrow_mut().reverse();
        } else {
            panic!("Can only use list.reverse on Lists");
        }

        Value::NIL
    }

    fn fill(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [value, list] = get_args!(args, 2);

        if let Value::List(inner) = list {
            inner.values.borrow_mut().fill(value);
        } else {
            panic!("Can only use list.fill on Lists");
        }

        Value::NIL
    }

    fn rep(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [value, list] = get_args!(args, 2);

        if let Value::List(inner) = list {
            if let Value::Number(n) = value {
                return Value::List(TList::new(rc!(RefCell::new(
                    std::iter::repeat(&*inner.values.borrow())
                        .take(n as usize)
                        .flatten()
                        .cloned()
                        .collect()
                ))));
            } else {
                panic!("Can only list.repeat with a number")
            }
        } else {
            panic!("Can only use list.repeat on Lists");
        }
    }

    fn push_n(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [list, value, num] = get_args!(args, 3);

        if let Value::List(inner) = list {
            if let Value::Number(n) = num {
                inner
                    .values
                    .borrow_mut()
                    .extend((0..n as usize).map(|_| value.clone()));
            } else {
                panic!("list.push_n requires a `number` of times as first argument")
            }
        } else {
            panic!("Can only use list.push_n on Lists");
        }

        Value::NIL
    }
}

// LIBRARY
impl Library for ListLib {
    fn get_name(&self) -> &str {
        "list"
    }

    fn get_function(&self, name: u64) -> Box<dyn Fn(&mut VM, Vec<Value>) -> Value> {
        match name {
            x if x == hash_u64!("len") => return Box::new(Self::len),
            x if x == hash_u64!("push") => return Box::new(Self::push),
            x if x == hash_u64!("insert") => return Box::new(Self::insert),
            x if x == hash_u64!("remove") => return Box::new(Self::remove),
            x if x == hash_u64!("map") => return Box::new(Self::map),
            x if x == hash_u64!("pop") => return Box::new(Self::pop),
            x if x == hash_u64!("clear") => return Box::new(Self::clear),
            x if x == hash_u64!("append") => return Box::new(Self::append),
            x if x == hash_u64!("concat") => return Box::new(Self::concat),
            x if x == hash_u64!("copy") => return Box::new(Self::copy),
            x if x == hash_u64!("count") => return Box::new(Self::count),
            x if x == hash_u64!("sort") => return Box::new(Self::sort),
            x if x == hash_u64!("reverse") => return Box::new(Self::reverse),
            x if x == hash_u64!("fill") => return Box::new(Self::fill),
            x if x == hash_u64!("rep") => return Box::new(Self::rep),
            x if x == hash_u64!("push_n") => return Box::new(Self::push_n),

            _ => panic!("Unknown function `{name}` on lib {}", self.get_name()),
        }
    }
}
