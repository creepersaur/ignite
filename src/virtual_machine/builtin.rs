use std::cell::RefCell;

use crate::{
    rc,
    virtual_machine::{types::string::TString, value::Value, vm::VM},
};

pub const BUILTINS: [&str; 3] = ["print", "println", "typeof"];

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

    vm.stack.push(Value::NIL);
}

pub fn builtin_typeof(vm: &mut VM) {
    let value = vm.pop();

    vm.stack
        .push(Value::String(TString(rc!(RefCell::new(value.get_type())))));
}
