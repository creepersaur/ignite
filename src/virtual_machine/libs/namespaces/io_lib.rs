use crate::virtual_machine::{libs::lib::Library, types::string::TString, value::Value, vm::VM};
use crossterm::event::{self, Event, KeyEvent};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io::{Read, Write, stdin, stdout};

pub struct IOLib;

impl IOLib {
    // Input
    fn read_line(_vm: &mut VM, args: Vec<Value>) -> Value {
        if args.len() > 0 {
            let msg = args
                .iter()
                .map(|x| x.to_string(false))
                .rev()
                .collect::<Vec<_>>()
                .join(" ");

            print!("{}", msg);
            let _ = stdout().flush();
        }

        let mut buf = String::new();
        stdin()
            .read_line(&mut buf)
            .expect("Couldn't read_line() from console");

        Value::String(TString::from_str(buf.trim()))
    }

    fn read_line_raw(_vm: &mut VM, args: Vec<Value>) -> Value {
        if args.len() > 0 {
            let msg = args
                .iter()
                .map(|x| x.to_string(false))
                .rev()
                .collect::<Vec<_>>()
                .join(" ");

            print!("{}", msg);
            let _ = stdout().flush();
        }

        let mut buf = String::new();
        stdin()
            .read_line(&mut buf)
            .expect("Couldn't read_line_raw() from console");

        Value::String(TString::new(buf))
    }

    fn read(_vm: &mut VM, args: Vec<Value>) -> Value {
        if args.len() > 0 {
            let msg = args
                .iter()
                .map(|x| x.to_string(false))
                .rev()
                .collect::<Vec<_>>()
                .join(" ");

            print!("{}", msg);
            let _ = stdout().flush();
        }

        let mut buf = [0u8; 1];

        stdin()
            .read_exact(&mut buf)
            .expect("Couldn't read() from console");

        Value::Char(buf[0] as char)
    }

    fn read_key(_vm: &mut VM, args: Vec<Value>) -> Value {
        if args.len() > 0 {
            let msg = args
                .iter()
                .map(|x| x.to_string(false))
                .rev()
                .collect::<Vec<_>>()
                .join(" ");

            print!("{}", msg);
            let _ = stdout().flush();
        }

        enable_raw_mode().unwrap();

        loop {
            // Block until a terminal event is available
            if let Event::Key(KeyEvent { code, .. }) = event::read().expect("Could not read key") {
                disable_raw_mode().unwrap();

                return Value::string(code);
            }
        }
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

    fn flush(_vm: &mut VM, _args: Vec<Value>) -> Value {
        let _ = stdout().flush();

        Value::NIL
    }

    pub fn write_fast(args: &[Value]) -> Value {
        let mut out = stdout().lock();
        for (i, x) in args.iter().rev().enumerate() {
            if i > 0 {
                out.write(b" ").unwrap();
            }
            out.write_all(x.to_string(false).as_bytes()).unwrap();
        }

        Value::NIL
    }

    pub fn write_line_fast(args: &[Value]) -> Value {
        let mut out = stdout().lock();
        for (i, x) in args.iter().rev().enumerate() {
            if i > 0 {
                out.write(b" ").unwrap();
            }
            out.write_all(x.to_string(false).as_bytes()).unwrap();
        }
        out.write(b"\n").unwrap();

        Value::NIL
    }

    pub fn write(vm: &mut VM, args: Vec<Value>) -> Value {
        let mut out = stdout().lock();
        for (i, x) in args.iter().rev().enumerate() {
            if i > 0 {
                out.write(b" ").unwrap();
            }
            if let Value::ClassObject(obj) = x {
                if let Some(Value::Function(f)) = obj
                    .base
                    .borrow()
                    .functions
                    .borrow()
                    .get(&hash_u64!("__tostring__"))
                {
                    vm.stack.push(x.clone());
                    vm.call_function(*f.clone(), 1);
                    vm.run(false, true);

                    let str = vm.pop();
                    out.write_all(str.to_string(false).as_bytes()).unwrap();
                }
            } else {
                out.write_all(x.to_string(false).as_bytes()).unwrap();
            }
        }

        Value::NIL
    }

    pub fn write_line(vm: &mut VM, args: Vec<Value>) -> Value {
        let mut out = stdout().lock();
        for (i, x) in args.iter().rev().enumerate() {
            if i > 0 {
                out.write(b" ").unwrap();
            }
            if let Value::ClassObject(obj) = x {
                if let Some(Value::Function(f)) = obj
                    .base
                    .borrow()
                    .functions
                    .borrow()
                    .get(&hash_u64!("__tostring__"))
                {
                    vm.stack.push(x.clone());
                    vm.call_function(*f.clone(), 1);
                    vm.run(false, true);

                    let str = vm.pop();
                    out.write_all(str.to_string(false).as_bytes()).unwrap();
                }
            } else {
                out.write_all(x.to_string(false).as_bytes()).unwrap();
            }
        }
        out.write(b"\n").unwrap();

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
            x if x == hash_u64!("read_line") => boxed!(Self::read_line),
            x if x == hash_u64!("read_line_raw") => boxed!(Self::read_line_raw),
            x if x == hash_u64!("read") => boxed!(Self::read),
            x if x == hash_u64!("read_key") => boxed!(Self::read_key),

            // OUTPUT
            x if x == hash_u64!("clear") => boxed!(Self::clear),
            x if x == hash_u64!("reset") => boxed!(Self::reset),
            x if x == hash_u64!("flush") => boxed!(Self::flush),
            x if x == hash_u64!("write") => boxed!(Self::write),
            x if x == hash_u64!("write_line") => boxed!(Self::write_line),

            _ => panic!("Unknown function `{name}` on lib {}", self.get_name()),
        }
    }
}
