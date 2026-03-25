use crate::{
    rc,
    virtual_machine::{
        builtin::*,
        chunk::Chunk,
        inst::Inst,
        libs::{
            dict_lib::DictLib, lib::Library, list_lib::ListLib, string_lib::StringLib,
            tuple_lib::TupleLib,
        },
        traits::member_accessible::IMemberAccessible,
        types::{dict::TDict, function::TFunction, list::TList, string::TString},
        value::Value,
    },
};
use bincode::config;
use simply_colored::*;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

const ORANGE: &str = "\x1b[38;2;255;150;60m";

pub struct VM {
    pub pos: usize,
    pub instructions: Vec<Inst>,
    pub stack: Vec<Value>,
    pub call_stack: Vec<usize>,
    pub scope_stack: Vec<usize>,
    pub constants: Vec<Value>,
    pub globals: HashMap<Rc<String>, (Value, bool)>,
    pub locals: Vec<HashMap<Rc<String>, (Value, bool)>>,
    pub libraries: HashMap<Rc<String>, Box<dyn Library>>,
    pub iterators: Vec<(Value, usize)>,
}

#[allow(unused)]
impl VM {
    pub fn new() -> Self {
        let mut libs: HashMap<_, Box<dyn Library>> = HashMap::new();
        libs.insert(rc!("string".to_string()), Box::new(StringLib));
        libs.insert(rc!("list".to_string()), Box::new(ListLib));
        libs.insert(rc!("tuple".to_string()), Box::new(TupleLib));
        libs.insert(rc!("dict".to_string()), Box::new(DictLib));

        Self {
            pos: 0,
            instructions: vec![],
            stack: Vec::with_capacity(100),
            call_stack: Vec::with_capacity(100),
            scope_stack: vec![],
            constants: Vec::with_capacity(100),
            globals: HashMap::new(),
            locals: vec![HashMap::new()],
            libraries: libs,
            iterators: vec![],
        }
    }

    pub fn advance(&mut self) {
        self.pos += 1;
    }

    pub fn push_inst(&mut self, instruction: Inst) {
        self.instructions.push(instruction);
    }

    #[inline]
    pub fn pop(&mut self) -> Value {
        return self.stack.pop().unwrap();
    }

    #[inline]
    pub fn pop_two(&mut self) -> (Value, Value) {
        let right = self
            .stack
            .pop()
            .expect("Stack underflow: missing right operand");
        let left = self
            .stack
            .pop()
            .expect("Stack underflow: missing left operand");

        (left, right)
    }

    #[allow(unused)]
    pub fn push_constants(&mut self, constants: Vec<Value>) {
        self.constants.extend(constants);
    }

    pub fn call_function(&mut self, f: TFunction) {
        if let Some((library, method)) = f.handler {
            if let Some(lib) = self.libraries.get(&library) {
                if let Some(this) = f.this {
                    self.stack.push(*this);
                }

                let value = lib.get_function(method)(self);
                self.stack.push(value);
            }
        } else {
            self.call_stack.push(self.pos);
            self.scope_stack.push(self.locals.len());
            self.pos = f.entry;
        }
    }

    pub fn print_instructions(&self) {
        let mut depth: i32 = 0;

        for (i, v) in self.instructions.iter().enumerate() {
            if let Inst::COMMENT(x) = v {
                println!(
                    "{BLACK}  \t{}--- {x} ---{RESET}",
                    format!(
                        "{}{}",
                        if depth < 0 { RED } else { DIM_BLACK },
                        "|  ".repeat(depth.abs() as usize)
                    ),
                );
                continue;
            }

            let s = format!("{:?}", v);

            // Split the first word (the opcode) from the rest
            let mut parts = s.splitn(2, '(');
            let opcode = parts.next().unwrap();
            let rest = parts.next().map_or("", |r| r);

            if let Inst::POP_SCOPE = v {
                depth -= 1;
            }

            if rest.is_empty() {
                print!(
                    "{MAGENTA}{:>2}{RESET}\t{}",
                    i,
                    format!(
                        "{}{}",
                        if depth < 0 { RED } else { DIM_BLACK },
                        "|  ".repeat(depth.abs() as usize)
                    )
                );
                if matches!(v, Inst::EXIT) | matches!(v, Inst::RETURN) {
                    print!("{RED}");
                } else if matches!(v, Inst::LIST(_))
                    | matches!(v, Inst::TUPLE(_))
                    | matches!(v, Inst::DICT(_))
                    | matches!(v, Inst::RANGE)
                {
                    print!("{GREEN}");
                } else if matches!(v, Inst::NOP) {
                    print!("{BLACK}")
                } else {
                    print!("{ORANGE}");
                }
                print!("{opcode}{RESET}");
            } else {
                print!(
                    "{MAGENTA}{:>2}{RESET}\t{BLACK}{}",
                    i,
                    format!(
                        "{}{}",
                        if depth < 0 { RED } else { DIM_BLACK },
                        "|  ".repeat(depth.abs() as usize)
                    ),
                );
                if matches!(v, Inst::EXIT) | matches!(v, Inst::RETURN) {
                    print!("{RED}");
                } else if matches!(v, Inst::LIST(_))
                    | matches!(v, Inst::TUPLE(_))
                    | matches!(v, Inst::DICT(_))
                    | matches!(v, Inst::RANGE)
                {
                    print!("{GREEN}");
                } else if matches!(v, Inst::NOP) {
                    print!("{BLACK}")
                } else {
                    print!("{ORANGE}");
                }
                print!("{opcode}{RESET}({BLUE}{}{RESET})", &rest[0..rest.len() - 1]);
            }

            if let Inst::LOAD_CONST(x) = v {
                print!("{BLACK}   {:>3?}{RESET}", self.constants[*x]);
            }
            if let Inst::PUSH_SCOPE = v {
                print!(" {{");
                depth += 1;
            }
            if let Inst::POP_SCOPE = v {
                println!(" }}");
            } else {
                println!("")
            }
        }
    }

    pub fn to_chunk(&self) -> Chunk {
        Chunk::new(self.constants.clone(), self.instructions.clone())
    }

    pub fn read_bytecode_file(&mut self, path: &str) {
        let bytecode_file = std::fs::read(path).unwrap();

        let decoded: (Chunk, _) =
            bincode::decode_from_slice(&bytecode_file, config::standard()).unwrap();

        self.constants = decoded.0.constants;
        self.instructions = decoded.0.instructions;
    }

    pub fn write_bytecode_file(&mut self, path: &str) {
        let chunk = Chunk::new(self.constants.clone(), self.instructions.clone());
        let encoded = bincode::encode_to_vec(chunk, config::standard()).unwrap();

        std::fs::write(path, encoded).unwrap();
    }

    pub fn run(&mut self, debug: bool, stop_at_return: bool) {
        while self.pos < self.instructions.len() {
            if debug {
                println!("{BLACK}{} ...{RESET}", self.pos);
            }
            let current = &self.instructions[self.pos];

            match current {
                Inst::EXIT => return,
                Inst::NOP => {}
                Inst::COMMENT(_) => {}
                Inst::PRINT => println!("{}", self.pop().to_string(false)),
                Inst::POP => {
                    self.pop();
                }
                Inst::DEFAULT => {
                    let default_value = self.pop();
                    let given_value = self.pop();

                    if let Value::NIL = given_value {
                        self.stack.push(default_value);
                    } else {
                        self.stack.push(given_value);
                    }
                }
                Inst::DEFAULT_NIL => {
                    if self.stack.len() == 0 {
                        self.stack.push(Value::NIL)
                    }
                }
                Inst::PUSH(value) => self.stack.push(value.clone()),
                Inst::DUP => self.stack.push(
                    self.stack
                        .last()
                        .expect("Cannot DUP, stack underflow.")
                        .clone(),
                ),

                // Collections
                Inst::LIST(length) => {
                    let values = (0..*length).map(|_| self.pop()).collect();
                    self.stack
                        .push(Value::List(TList::new(rc!(RefCell::new(values)))));
                }
                Inst::TUPLE(length) => {
                    let values = (0..*length).map(|_| self.pop()).collect();
                    self.stack
                        .push(Value::Tuple(TList::new_tuple(rc!(RefCell::new(values)))));
                }
                Inst::DICT(length) => {
                    let values = (0..*length).map(|_| self.pop_two()).collect();
                    self.stack
                        .push(Value::Dict(TDict::new(rc!(RefCell::new(values)))));
                }

                Inst::RANGE => {
                    let inclusive_val = self.pop();
                    let step = self.pop();
                    let end = self.pop();
                    let start = self.pop();

                    let inclusive = if let Value::Bool(x) = inclusive_val {
                        x
                    } else {
                        false
                    };

                    self.stack.push(Value::Range {
                        start: Box::new(start),
                        end: Box::new(end),
                        step: Box::new(step),
                        inclusive,
                    });
                }

                Inst::ADD => {
                    let (a, b) = &self.pop_two();

                    if let (Value::Number(a), Value::Number(b)) = (a, b) {
                        self.stack.push(Value::Number(a + b));
                    } else if let (Value::String(a), Value::String(b)) = (a, b) {
                        self.stack
                            .push(Value::String(TString(rc!(RefCell::new(format!(
                                "{}{}",
                                a.to_string(),
                                b.to_string()
                            ))))));
                    } else if let (Value::String(a), Value::Char(b)) = (a, b) {
                        self.stack
                            .push(Value::String(TString(rc!(RefCell::new(format!(
                                "{}{}",
                                a.to_string(),
                                b
                            ))))));
                    } else if let (Value::Char(a), Value::String(b)) = (a, b) {
                        self.stack
                            .push(Value::String(TString(rc!(RefCell::new(format!(
                                "{}{}",
                                a,
                                b.to_string()
                            ))))));
                    } else {
                        panic!("Cannot add {} and {}", a.get_type(), b.get_type());
                    }
                }
                Inst::SUB => {
                    let (a, b) = &self.pop_two();

                    if let (Value::Number(a), Value::Number(b)) = (a, b) {
                        self.stack.push(Value::Number(a - b));
                    } else {
                        panic!("Cannot subtract {} by {}", a.get_type(), b.get_type());
                    }
                }
                Inst::MUL => {
                    let (a, b) = &self.pop_two();

                    if let (Value::Number(a), Value::Number(b)) = (a, b) {
                        self.stack.push(Value::Number(a * b));
                    } else if let (Value::String(a), Value::Number(b)) = (a, b) {
                        self.stack.push(Value::String(TString(Rc::new(RefCell::new(
                            a.0.borrow().repeat(*b as usize),
                        )))));
                    } else {
                        panic!("Cannot multiply `{}` with `{}`", a.get_type(), b.get_type());
                    }
                }
                Inst::DIV => {
                    let (a, b) = &self.pop_two();

                    if let (Value::Number(a), Value::Number(b)) = (a, b) {
                        if *b == 0.0 {
                            panic!("Cannot divide by Zero");
                        } else {
                            self.stack.push(Value::Number(a / b));
                        }
                    } else {
                        panic!("Cannot divide `{}` by `{}`", a.get_type(), b.get_type());
                    }
                }
                Inst::POW => {
                    let (a, b) = &self.pop_two();

                    if let (Value::Number(a), Value::Number(b)) = (a, b) {
                        self.stack.push(Value::Number(a.powf(*b)));
                    } else {
                        panic!("Cannot POW `{}` and `{}`", a.get_type(), b.get_type());
                    }
                }
                Inst::MOD => {
                    let (a, b) = &self.pop_two();

                    if let (Value::Number(a), Value::Number(b)) = (a, b) {
                        self.stack.push(Value::Number(a % b));
                    } else {
                        panic!("Cannot MOD `{}` and `{}`", a.get_type(), b.get_type());
                    }
                }

                Inst::NEG => {
                    let num = self.pop().as_number();
                    self.stack.push(Value::Number(-num));
                }
                Inst::POS => {
                    let num = self.pop().as_number();
                    self.stack.push(Value::Number(num));
                }

                Inst::EQ => {
                    let result = match self.pop_two() {
                        (Value::Number(x), Value::Number(y)) => x == y,
                        (Value::Bool(x), Value::Bool(y)) => x == y,
                        (Value::Char(x), Value::Char(y)) => x == y,

                        (Value::String(x), Value::String(y)) => {
                            Rc::ptr_eq(&x.0, &y.0) || *x.0.borrow() == *y.0.borrow()
                        }

                        _ => false,
                    };

                    self.stack.push(Value::Bool(result));
                }
                Inst::NEQ => {
                    let result = match self.pop_two() {
                        (Value::Number(x), Value::Number(y)) => x != y,
                        (Value::Bool(x), Value::Bool(y)) => x != y,
                        (Value::Char(x), Value::Char(y)) => x != y,

                        (Value::String(x), Value::String(y)) => {
                            !Rc::ptr_eq(&x.0, &y.0) || *x.0 != *y.0
                        }

                        _ => panic!("Cannot NEQ"),
                    };

                    self.stack.push(Value::Bool(result));
                }
                Inst::GT => {
                    if let (Value::Number(a), Value::Number(b)) = self.pop_two() {
                        self.stack.push(Value::Bool(a > b));
                    } else {
                        panic!("GT expects numbers");
                    }
                }
                Inst::LT => {
                    if let (Value::Number(a), Value::Number(b)) = self.pop_two() {
                        self.stack.push(Value::Bool(a < b));
                    } else {
                        panic!("LT expects numbers");
                    }
                }
                Inst::GE => {
                    if let (Value::Number(a), Value::Number(b)) = self.pop_two() {
                        self.stack.push(Value::Bool(a >= b));
                    } else {
                        panic!("GE expects numbers");
                    }
                }
                Inst::LE => {
                    if let (Value::Number(a), Value::Number(b)) = self.pop_two() {
                        self.stack.push(Value::Bool(a <= b));
                    } else {
                        panic!("LE expects numbers");
                    }
                }
                Inst::AND => {
                    let (a, b) = self.pop_two();

                    self.stack.push(Value::Bool(a.is_truthy() && b.is_truthy()));
                }
                Inst::OR => {
                    let (a, b) = self.pop_two();

                    self.stack.push(Value::Bool(a.is_truthy() || b.is_truthy()));
                }
                Inst::NOT => {
                    let res = self.pop().is_truthy();
                    self.stack.push(Value::Bool(!res));
                }

                Inst::LOAD_CONST(idx) => self.stack.push(self.constants[*idx].clone()),
                Inst::STORE_GLOBAL(name) => {
                    let name = name.clone();
                    let value = self.pop();
                    self.globals.insert(name, (value, false));
                }
                Inst::LOAD_GLOBAL(name) => {
                    self.stack.push(
                        self.globals
                            .get(name)
                            .expect("Global `{name}` doesn't exist.")
                            .0
                            .clone(),
                    );
                }

                Inst::PUSH_SCOPE => self.locals.push(HashMap::new()),
                Inst::POP_SCOPE => {
                    self.locals.pop();
                }
                Inst::STORE_LOCAL(name) => {
                    let name = name.clone();
                    let value = self.pop();
                    self.locals.last_mut().unwrap().insert(name, (value, false));
                }
                Inst::LOAD_LOCAL(name) => {
                    let mut found = None;

                    for scope in self.locals.iter().rev() {
                        if let Some(val) = scope.get(name) {
                            found = Some(val.clone());
                            break;
                        }
                    }

                    if let Some((val, _)) = found {
                        self.stack.push(val);
                    } else {
                        panic!("Unknown local variable: {name}");
                    }
                }
                Inst::STORE_LOCAL_CONST(name) => {
                    let name = name.clone();
                    let value = self.pop();
                    self.locals.last_mut().unwrap().insert(name, (value, true));
                }

                Inst::LOAD(name) => {
                    let mut found = None;

                    for scope in self.locals.iter().rev() {
                        if let Some(val) = scope.get(name) {
                            found = Some(val.clone());
                            break;
                        }
                    }

                    if let Some((val, _)) = found {
                        self.stack.push(val);
                    } else if let Some((val, _)) = self.globals.get(name) {
                        self.stack.push(val.clone());
                    } else {
                        panic!("Unknown local/global variable: {name}");
                    }
                }
                Inst::SET_VAR(name) => {
                    let mut found_idx = None;

                    for i in (0..self.locals.len()).rev() {
                        if let Some((_, is_const)) = self.locals[i].get(name) {
                            if *is_const {
                                panic!("Cannot set a constant `{name}`");
                            } else {
                                found_idx = Some(i);
                            }
                            break;
                        }
                    }
                    if let Some(scope) = found_idx {
                        let name = name.clone();
                        let value = self.pop();
                        self.locals[scope].insert(name, (value, false));
                    } else if let Some((_, is_const)) = self.globals.get(name) {
                        if *is_const {
                            panic!("Cannot set a global constant `{name}`");
                        } else {
                            let name = name.clone();
                            let value = self.pop();
                            self.globals.insert(name, (value, false));
                        }
                    }
                }

                Inst::JUMP(idx) => {
                    self.pos = *idx;
                    continue;
                }
                Inst::JUMP_IF_FALSE(idx) => {
                    let idx = *idx;
                    if !self.pop().is_truthy() {
                        self.pos = idx;
                        continue;
                    }
                }

                Inst::CALL => {
                    let func = self.pop();

                    if let Value::Function(f) = func {
                        self.call_function(f);
                    } else {
                        panic!("Tried calling non-function: {func:?}")
                    }
                }
                Inst::CALL_BUILTIN(name, arg_count) => match &***name {
                    "print" => builtin_print(self, *arg_count, false),
                    "println" => builtin_print(self, *arg_count, true),
                    "typeof" => builtin_typeof(self),
                    "round" => builtin_round(self),
                    _ => panic!("Unknown built-in: {name}"),
                },
                Inst::RETURN => {
                    if let Some(last) = self.call_stack.last() {
                        self.pos = *last;
                        self.call_stack.pop();

                        if let Some(depth) = self.scope_stack.pop() {
                            self.locals.truncate(depth);
                        }
                    } else {
                        break;
                    }
                    if stop_at_return {
                        break;
                    }
                }

                Inst::GET_PROP => {
                    let member = self.pop();
                    let target = self.pop();

                    match target {
                        Value::String(x) => {
                            let value = x.get_member(self, &member);
                            self.stack.push(value);
                        }

                        Value::List(x) => {
                            let value = x.get_member(self, &member);
                            self.stack.push(value);
                        }

                        Value::Tuple(x) => {
                            let value = x.get_member(self, &member);
                            self.stack.push(value);
                        }

                        Value::Dict(x) => {
                            let value = x.get_member(self, &member);
                            self.stack.push(value);
                        }

                        _ => panic!("Cannot get property on `{target:?}`"),
                    }
                }
                Inst::SET_PROP => {
                    let member = self.pop();
                    let target = self.pop();
                    let value = self.pop();

                    match target {
                        Value::List(x) => {
                            x.set_member(&member, value);
                        }

                        Value::Tuple(x) => {
                            x.set_member(&member, value);
                        }

                        Value::Dict(x) => {
                            x.set_member(&member, value);
                        }

                        _ => panic!("Cannot set property `{member:?}` on `{target:?}`"),
                    }
                }

                Inst::GET_ITER => {
                    let value = self.pop();

                    if let Value::String(s) = &value {
                        let chars: Vec<Value> =
                            s.0.borrow().chars().map(|c| Value::Char(c)).collect();
                        self.iterators
                            .push((Value::List(TList::new_tuple(rc!(RefCell::new(chars)))), 0)); // or a dedicated variant
                    } else {
                        self.iterators.push((value, 0));
                    }
                }
                Inst::FOR_ITER(jump_end) => {
                    let target_idx = self.iterators.len() - 1;
                    let (value, idx) = &mut self.iterators[target_idx];

                    if let Value::List(list) = value {
                        if *idx < list.values.borrow().len() {
                            self.stack.push(list.values.borrow()[*idx].clone());
                            *idx += 1;
                        } else {
                            self.iterators.pop();
                            self.pos = *jump_end;
                            continue;
                        }
                    } else if let Value::Tuple(list) = value {
                        if *idx < list.values.borrow().len() {
                            self.stack.push(list.values.borrow()[*idx].clone());
                            *idx += 1;
                        } else {
                            self.iterators.pop();
                            self.pos = *jump_end;
                            continue;
                        }
                    } else if let Value::Range {
                        start,
                        end,
                        step,
                        inclusive,
                    } = value
                    {
                        let s = start.as_number();
                        let e = end.as_number();
                        let step_val = step.as_number();

                        let current_value = s + (*idx as f64 * step_val);

                        let is_within_bounds = if *inclusive {
                            if step_val >= 0.0 {
                                current_value <= e
                            } else {
                                current_value >= e
                            }
                        } else {
                            if step_val >= 0.0 {
                                current_value < e
                            } else {
                                current_value > e
                            }
                        };

                        if is_within_bounds {
                            self.stack.push(Value::Number(current_value));
                            *idx += 1;
                        } else {
                            // Range exhausted
                            self.iterators.pop();
                            self.pos = *jump_end;
                            continue;
                        }
                    } else {
                        panic!("Cannot iterate over {value:?}");
                    }
                }

                _ => panic!("Unimplemented instruction: {current:?}"),
            }

            self.advance();
        }
    }
}
