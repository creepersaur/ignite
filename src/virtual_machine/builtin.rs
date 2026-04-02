use crate::virtual_machine::{types::string::TString, value::Value, vm::VM};

pub const BUILTINS: [&str; 8] = ["typeof", "round", "string", "number", "bool", "char", "print", "println"];

pub fn builtin_typeof(vm: &mut VM) {
    let value = vm.pop();

    vm.stack.push(Value::String(TString::new(value.get_type())));
}

// TYPES

pub fn builtin_string(vm: &mut VM) {
    let value = vm.pop();

    vm.stack
        .push(Value::String(TString::new(value.to_string(false))));
}

pub fn builtin_number(vm: &mut VM) {
    let value = vm.pop();

    if let Value::String(x) = value {
        vm.stack.push(Value::Number(
            x.0.parse::<f64>()
                .expect("Failed to convert string to number"),
        ));
    } else if let Value::Char(x) = value {
        vm.stack.push(Value::Number(x as u64 as f64));
    } else if let Value::Bool(x) = value {
        vm.stack.push(Value::Number(x as u64 as f64));
    } else if let Value::NIL = value {
        vm.stack.push(Value::Number(0.0));
    } else {
        vm.stack.push(Value::Number(value.as_number()));
    }
}

pub fn builtin_bool(vm: &mut VM) {
    let value = vm.pop();

    if let Value::Number(x) = value
        && x == 0.0
    {
        vm.stack.push(Value::Bool(false));
        return;
    }

    vm.stack.push(Value::Bool(value.is_truthy()));
}

pub fn builtin_char(vm: &mut VM) {
    let value = vm.pop();

    if let Value::Number(x) = value {
        vm.stack
            .push(Value::Char(char::from_u32(x as u32).unwrap()));
    } else if matches!(value, Value::Char(_)) {
        vm.stack.push(value);
    } else if let Value::String(v) = value {
        vm.stack.push(Value::Char(v.0.chars().nth(0).unwrap()));
    } else {
        panic!("Cannot convert {value:?} to char");
    }
}
