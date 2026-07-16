use crate::{
    get_args,
    virtual_machine::{libs::lib::Library, value::Value, vm::VM},
};

pub struct RandomLib;

impl RandomLib {
    fn int(_vm: &mut VM, _args: Vec<Value>) -> Value {
        Value::Number(rand::random::<i64>() as f64)
    }

    fn uint(_vm: &mut VM, _args: Vec<Value>) -> Value {
        Value::Number(rand::random::<u64>() as f64)
    }

    fn int_range(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [start, end] = get_args!(args, 2);

        let start = start.as_number("number convertion");

        Value::Number(
            (rand::random::<f64>() * (end.as_number("number convertion") - start) + start).floor(),
        )
    }

    fn float(_vm: &mut VM, _args: Vec<Value>) -> Value {
        Value::Number(rand::random())
    }

    fn float_range(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [start, end] = get_args!(args, 2);

        let start = start.as_number("number convertion");

        Value::Number(rand::random::<f64>() * (end.as_number("number convertion") - start) + start)
    }
}

// LIBRARY
impl Library for RandomLib {
    fn get_name(&self) -> &str {
        "Random"
    }

    fn get_function(&self, name: u64) -> Box<dyn Fn(&mut VM, Vec<Value>) -> Value> {
        match name {
            // INPUT
            x if x == hash_u64!("int") => boxed!(Self::int),
            x if x == hash_u64!("uint") => boxed!(Self::uint),
            x if x == hash_u64!("int_range") => boxed!(Self::int_range),

            x if x == hash_u64!("float") => boxed!(Self::float),
            x if x == hash_u64!("float_range") => boxed!(Self::float_range),

            _ => panic!("Unknown function `{name}` on lib {}", self.get_name()),
        }
    }
}
