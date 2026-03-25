use std::{cell::RefCell, cmp::Ordering, rc::Rc};

use crate::{
    misc::to_index::to_index,
    rc,
    virtual_machine::{libs::lib::Library, types::list::TList, value::Value, vm::VM},
};

pub struct ListLib;

impl ListLib {
    fn len(vm: &mut VM) -> Value {
        let list = vm.pop();

        if let Value::List(inner) = list {
            Value::Number(inner.values.borrow().len() as f32)
        } else {
            panic!("Can only use list.len on Lists");
        }
    }

    fn push(vm: &mut VM) -> Value {
        let list = vm.pop();
        let new_value = vm.pop();

        if let Value::List(x) = list {
            let values = x.values;
            values.borrow_mut().push(new_value);
        } else {
            panic!("Can only use list.push on Lists")
        }

        Value::NIL
    }

    fn insert(vm: &mut VM) -> Value {
        let list = vm.pop();
        let new_value = vm.pop();
        let index = vm.pop();

        if let Value::List(x) = list {
            let values = x.values;
            values
                .borrow_mut()
                .insert(index.as_number() as usize, new_value);
        } else {
            panic!("Can only use list.insert on Lists");
        }

        Value::NIL
    }

    fn remove(vm: &mut VM) -> Value {
        let (index, list) = vm.pop_two();

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

    fn pop(vm: &mut VM) -> Value {
        let list = vm.pop();

        if let Value::List(inner) = list {
            return inner.values.borrow_mut().pop().unwrap_or(Value::NIL);
        } else {
            panic!("Can only use list.pop on Lists");
        }
    }

    fn clear(vm: &mut VM) -> Value {
        let list = vm.pop();

        if let Value::List(inner) = list {
            inner.values.borrow_mut().clear();
            return Value::NIL;
        } else {
            panic!("Can only use list.clear on Lists");
        }
    }

    fn map(vm: &mut VM) -> Value {
        let (func, list) = vm.pop_two();
        let mut new_array = vec![];

        if let Value::List(inner) = list {
            if let Value::Function(f) = func {
                for i in inner.values.borrow().iter() {
                    vm.stack.push(i.clone());
                    vm.call_function(f.clone());
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

    fn append(vm: &mut VM) -> Value {
        let (other, list) = vm.pop_two();

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

    fn concat(vm: &mut VM) -> Value {
        let (other, list) = vm.pop_two();

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

    fn copy(vm: &mut VM) -> Value {
        let list = vm.pop();

        if let Value::List(inner) = list {
            Value::List(inner.clone())
        } else {
            panic!("Can only use list.copy on Lists");
        }
    }

    fn count(vm: &mut VM) -> Value {
        let (item, list) = vm.pop_two();

        if let Value::List(inner) = list {
            let count = inner.values.borrow().iter().filter(|x| x == &&item).count();
            Value::Number(count as f32)
        } else {
            panic!("Can only use list.count on Lists");
        }
    }

    fn sort(vm: &mut VM) -> Value {
        let list = vm.pop();

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

    fn reverse(vm: &mut VM) -> Value {
        let list = vm.pop();

        if let Value::List(inner) = list {
            inner.values.borrow_mut().reverse();
        } else {
            panic!("Can only use list.reverse on Lists");
        }

        Value::NIL
    }

    fn fill(vm: &mut VM) -> Value {
        let (value, list) = vm.pop_two();

        if let Value::List(inner) = list {
            inner.values.borrow_mut().fill(value);
        } else {
            panic!("Can only use list.reverse on Lists");
        }

        Value::NIL
    }

    fn rep(vm: &mut VM) -> Value {
        let (value, list) = vm.pop_two();

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

    fn push_n(vm: &mut VM) -> Value {
        let list = vm.pop();
        let value = vm.pop();
        let num = vm.pop();

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

    fn get_function(&self, name: Rc<String>) -> Box<dyn Fn(&mut VM) -> Value> {
        match name.as_str() {
            "len" => return Box::new(Self::len),
            "push" => return Box::new(Self::push),
            "insert" => return Box::new(Self::insert),
            "remove" => return Box::new(Self::remove),
            "map" => return Box::new(Self::map),
            "pop" => return Box::new(Self::pop),
            "clear" => return Box::new(Self::clear),
            "append" => return Box::new(Self::append),
            "concat" => return Box::new(Self::concat),
            "copy" => return Box::new(Self::copy),
            "count" => return Box::new(Self::count),
            "sort" => return Box::new(Self::sort),
            "reverse" => return Box::new(Self::reverse),
            "fill" => return Box::new(Self::fill),
            "rep" => return Box::new(Self::rep),
            "push_n" => return Box::new(Self::push_n),

            _ => panic!("Unknown function `{name}` on lib {}", self.get_name()),
        }
    }
}
