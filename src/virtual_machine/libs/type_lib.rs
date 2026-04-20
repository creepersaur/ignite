use crate::{
    get_args, hash_u64,
    virtual_machine::{libs::lib::Library, types::string::TString, value::Value, vm::VM},
};

pub struct TypeLib;

impl TypeLib {
    pub fn r#typeof(_vm: &mut VM, args: Vec<Value>) -> Value {
        let value = get_args!(args);

        Value::String(TString::new(value.get_type()))
    }

    // TYPES

    pub fn string(_vm: &mut VM, args: Vec<Value>) -> Value {
        let value = get_args!(args);

        if matches!(value, Value::String(_)) {
            value.clone()
        } else {
            Value::string(value)
        }
    }

    pub fn number(_vm: &mut VM, args: Vec<Value>) -> Value {
        let value = get_args!(args);

        if let Value::String(x) = value {
            Value::Number(
                x.0.parse::<f64>()
                    .expect("Failed to convert string to number"),
            )
        } else if let Value::Char(x) = value {
            Value::Number(*x as u64 as f64)
        } else if let Value::Bool(x) = value {
            Value::Number(*x as u64 as f64)
        } else if let Value::NIL = value {
            Value::Number(0.0)
        } else {
            Value::Number(value.as_number())
        }
    }

    pub fn bool(_vm: &mut VM, args: Vec<Value>) -> Value {
        let value = get_args!(args);

        if let Value::Number(x) = value
            && *x == 0.0
        {
            Value::Bool(false)
        } else {
            Value::Bool(value.is_truthy())
        }
    }

    pub fn char(_vm: &mut VM, args: Vec<Value>) -> Value {
        let value = get_args!(args);

        if let Value::Number(x) = value {
            Value::Char(char::from_u32(*x as u32).unwrap())
        } else if matches!(value, Value::Char(_)) {
            value.clone()
        } else if let Value::String(v) = value {
            Value::Char(v.0.chars().nth(0).unwrap())
        } else {
            panic!("Cannot convert {value:?} to char");
        }
    }
}

impl Library for TypeLib {
    fn get_name(&self) -> &str {
        "type"
    }

    fn get_function(&self, name: u64) -> Box<dyn Fn(&mut VM, Vec<Value>) -> Value> {
        match name {
            name if hash_u64!("typeof") == name => Box::new(Self::r#typeof),

            name if hash_u64!("string") == name => Box::new(Self::r#string),
            name if hash_u64!("number") == name => Box::new(Self::r#number),
            name if hash_u64!("bool") == name => Box::new(Self::r#bool),
            name if hash_u64!("char") == name => Box::new(Self::r#char),

            _ => panic!("Unknown type lib function"),
        }
    }
}
