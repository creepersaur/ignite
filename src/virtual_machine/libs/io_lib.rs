use std::io::{Write, stdin, stdout};

use crate::{
    hash_u64,
    virtual_machine::{libs::lib::Library, types::string::TString, value::Value, vm::VM},
};

pub struct IOLib;

impl IOLib {
    // Geometry
    fn read_line(_vm: &mut VM, args: Vec<Value>) -> Value {
        let msg = args
            .iter()
            .map(|x| x.to_string(false))
            .collect::<Vec<_>>()
            .join(" ");

        print!("{}", msg);
        let _ = stdout().flush();

        let mut buf = String::new();
        stdin()
            .read_line(&mut buf)
            .expect("Couldn't read_line() from console");

        Value::String(TString::from_str(buf.trim()))
    }

    fn read_line_raw(_vm: &mut VM, args: Vec<Value>) -> Value {
        let msg = args
            .iter()
            .map(|x| x.to_string(false))
            .collect::<Vec<_>>()
            .join(" ");

        print!("{}", msg);
        let _ = stdout().flush();

        let mut buf = String::new();
        stdin()
            .read_line(&mut buf)
            .expect("Couldn't read_line_raw() from console");

        Value::String(TString::new(buf))
    }
}

// LIBRARY
impl Library for IOLib {
    fn get_name(&self) -> &str {
        "io"
    }

    fn get_function(&self, name: u64) -> Box<dyn Fn(&mut VM, Vec<Value>) -> Value> {
        match name {
            // INPUT
            x if x == hash_u64!("read_line") => Box::new(Self::read_line),
            x if x == hash_u64!("read_line_raw") => Box::new(Self::read_line_raw),

            _ => panic!("Unknown function `{name}` on lib {}", self.get_name()),
        }
    }
}
