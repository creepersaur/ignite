use crate::virtual_machine::{types::string::TString, value::Value, vm::VM};

pub const BUILTIN_VOIDS: [&str; 2] = ["print", "println"];
pub const BUILTINS: [&str; 6] = [
    "typeof", "round", "string", "number", "bool", "char",
];

pub fn builtin_print(vm: &mut VM, arg_count: usize, newline: bool) {
    let args = (0..arg_count).map(|_| vm.pop()).collect::<Vec<_>>();
    let string = args
        .iter()
        .rev()
        .map(|x| x.to_string(false))
        .collect::<Vec<_>>()
        .join(" ");

    if newline {
        println!("{string}");
    } else {
        print!("{string}");
    }
}

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

    vm.stack.push(Value::Number(value.as_number()));
}

pub fn builtin_bool(vm: &mut VM) {
    let value = vm.pop();

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
        vm.stack
            .push(Value::Char(v.0.chars().nth(0).unwrap()));
    } else {
        panic!("Cannot convert {value:?} to char");
    }
}
