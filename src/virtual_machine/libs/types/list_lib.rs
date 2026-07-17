use rand::seq::SliceRandom;

use crate::{
    get_args,
    misc::to_index::to_index,
    virtual_machine::{libs::lib::Library, types::list::TList, value::Value, vm::VM},
};
use std::{cell::RefCell, cmp::Ordering};

macro_rules! list_functions {
    ($($name:literal),* $(,)?) => {
        pub const LIST_FUNCTIONS: &[&str] = &[
            $($name),*
        ];

        pub const LIST_FUNCTION_IDS: &[u64] = &[
            $(hash_u64!($name)),*
        ];
    };
}

list_functions![
    "len",
    "push",
    "insert",
    "remove",
    "map",
    "map_enumerate",
    "pop",
    "clear",
    "append",
    "concat",
    "copy",
    "count",
    "sort",
    "reverse",
    "fill",
    "rep",
    "push_n",
    "shuffle",
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
        let [list, index, new_value] = get_args!(args, 3);

        if let Value::List(x) = list {
            x.values
                .borrow_mut()
                .insert(index.as_number("number key convertion") as usize, new_value);
        } else {
            panic!("Can only use list.insert on Lists");
        }

        Value::NIL
    }

    fn remove(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [list, index] = get_args!(args, 2);

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
        let [list, func] = get_args!(args, 2);
        let mut new_array = vec![];

        if let Value::List(inner) = list {
            if let Value::Function(f) = func {
                for i in inner.values.borrow().iter() {
                    vm.stack.push(i.clone());
                    vm.call_function(*f.clone(), 1);
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

    fn map_enumerate(vm: &mut VM, args: Vec<Value>) -> Value {
        let [list, func] = get_args!(args, 2);
        let mut new_array = vec![];

        if let Value::List(inner) = list {
            if let Value::Function(f) = func {
                for (i, v) in inner.values.borrow().iter().enumerate() {
                    vm.stack.push(v.clone());
                    vm.stack.push(Value::Number(i as f64));
                    vm.call_function(*f.clone(), 2);
                    vm.run(false, true);
                    let new_value = vm.pop();
                    new_array.push(new_value);
                }

                return Value::List(TList::new(rc!(RefCell::new(new_array))));
            } else {
                panic!("Expected function in list.map_enumerate");
            }
        } else {
            panic!("Can only use list.map_enumerate on Lists");
        }
    }

    fn append(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [list, other] = get_args!(args, 2);

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
        let [list, other] = get_args!(args, 2);

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
            Value::List(TList::from(inner.values.borrow().clone()))
        } else {
            panic!("Can only use list.copy on Lists");
        }
    }

    fn count(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [list, item] = get_args!(args, 2);

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
        let [list, value] = get_args!(args, 2);

        if let Value::List(inner) = list {
            inner.values.borrow_mut().fill(value);
        } else {
            panic!("Can only use list.fill on Lists");
        }

        Value::NIL
    }

    fn rep(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [list, value] = get_args!(args, 2);

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

    fn shuffle(_vm: &mut VM, args: Vec<Value>) -> Value {
        let list = get_args!(args);

        if let Value::List(inner) = list {
            let mut values = inner.values.borrow_mut();
            values.shuffle(&mut rand::rng());
        } else {
            panic!("Can only use list.shuffle on Lists");
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
            x if x == hash_u64!("len") => return boxed!(Self::len),
            x if x == hash_u64!("push") => return boxed!(Self::push),
            x if x == hash_u64!("insert") => return boxed!(Self::insert),
            x if x == hash_u64!("remove") => return boxed!(Self::remove),
            x if x == hash_u64!("map") => return boxed!(Self::map),
            x if x == hash_u64!("map_enumerate") => return boxed!(Self::map_enumerate),
            x if x == hash_u64!("pop") => return boxed!(Self::pop),
            x if x == hash_u64!("clear") => return boxed!(Self::clear),
            x if x == hash_u64!("append") => return boxed!(Self::append),
            x if x == hash_u64!("concat") => return boxed!(Self::concat),
            x if x == hash_u64!("copy") => return boxed!(Self::copy),
            x if x == hash_u64!("count") => return boxed!(Self::count),
            x if x == hash_u64!("sort") => return boxed!(Self::sort),
            x if x == hash_u64!("reverse") => return boxed!(Self::reverse),
            x if x == hash_u64!("fill") => return boxed!(Self::fill),
            x if x == hash_u64!("rep") => return boxed!(Self::rep),
            x if x == hash_u64!("push_n") => return boxed!(Self::push_n),
            x if x == hash_u64!("shuffle") => return boxed!(Self::shuffle),

            _ => panic!("Unknown function `{name}` on lib {}", self.get_name()),
        }
    }
}
