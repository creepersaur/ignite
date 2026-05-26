use std::io::{Read, Write, stdin, stdout};

use crate::virtual_machine::{libs::lib::Library, types::string::TString, value::Value, vm::VM};

pub struct IOLib;

impl IOLib {
    // Input
    fn read_line(_vm: &mut VM, args: Vec<Value>) -> Value {
        let msg = args
            .iter()
            .map(|x| x.to_string(false))
            .rev()
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
            .rev()
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

    fn read(_vm: &mut VM, args: Vec<Value>) -> Value {
        let msg = args
            .iter()
            .map(|x| x.to_string(false))
            .rev()
            .collect::<Vec<_>>()
            .join(" ");

        print!("{}", msg);
        let _ = stdout().flush();

        let mut buf = [0u8; 1];

        stdin()
            .read_exact(&mut buf)
            .expect("Couldn't read() from console");

        Value::Char(buf[0] as char)
    }

    // Output
    fn clear(_vm: &mut VM, _args: Vec<Value>) -> Value {
        print!("\x1B[2J\x1B[1;1H");

        Value::NIL
    }

    fn reset(_vm: &mut VM, _args: Vec<Value>) -> Value {
        print!("{esc}c", esc = 27 as char);

        Value::NIL
    }

    fn write(_vm: &mut VM, args: Vec<Value>) -> Value {
        let msg = args
            .iter()
            .map(|x| x.to_string(false))
            .rev()
            .collect::<Vec<_>>()
            .join(" ");

        print!("{msg}");
        let _ = stdout().flush();

        Value::NIL
    }

    fn write_line(_vm: &mut VM, args: Vec<Value>) -> Value {
        let msg = args
            .iter()
            .map(|x| x.to_string(false))
            .rev()
            .collect::<Vec<_>>()
            .join(" ");

        println!("{msg}");

        Value::NIL
    }
}

// LIBRARY
impl Library for IOLib {
    fn get_name(&self) -> &str {
        "IO"
    }

    fn get_function(&self, name: u64) -> Box<dyn Fn(&mut VM, Vec<Value>) -> Value> {
        match name {
            // INPUT
            x if x == hash_u64!("read_line") => Box::new(Self::read_line),
            x if x == hash_u64!("read_line_raw") => Box::new(Self::read_line_raw),
            x if x == hash_u64!("read") => Box::new(Self::read),

            // OUTPUT
            x if x == hash_u64!("clear") => Box::new(Self::clear),
            x if x == hash_u64!("reset") => Box::new(Self::reset),
            x if x == hash_u64!("write") => Box::new(Self::write),
            x if x == hash_u64!("write_line") => Box::new(Self::write_line),

            _ => panic!("Unknown function `{name}` on lib {}", self.get_name()),
        }
    }
}
