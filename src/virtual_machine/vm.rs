use crate::{
    hash_u64, lib_function, rc,
    virtual_machine::{
        chunk::Chunk,
        inst::Inst,
        libs::{
            lib::Library,
            namespaces::{io_lib::IOLib, math_lib::MathLib},
            type_lib::TypeLib,
            types::{
                dict_lib::DictLib, list_lib::ListLib, string_lib::StringLib, tuple_lib::TupleLib,
            },
        },
        namespaces::standard_namespace::load_standard_namespace,
        traits::member_accessible::IMemberAccessible,
        types::{
            dict::TDict, r#enum::TEnum, function::TFunction, list::TList, string::TString,
            r#struct::TStruct,
        },
        value::Value,
    },
};
use simply_colored::*;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

const ORANGE: &str = "\x1b[38;2;255;150;60m";

pub struct CallFrame {
    scope_base: usize,
    return_addr: usize,
    upvalues: Vec<Rc<RefCell<HashMap<u64, (Value, bool)>>>>,
}

pub struct VM {
    pub pos: usize,
    pub instructions: Vec<Inst>,
    pub stack: Vec<Value>,
    pub call_stack: Vec<CallFrame>,
    pub constants: Vec<Value>,
    pub globals: HashMap<u64, (Value, bool)>,
    pub locals: Vec<Rc<RefCell<HashMap<u64, (Value, bool)>>>>,
    pub libraries: HashMap<u64, Box<dyn Library>>,
    pub iterators: Vec<(Value, usize)>,
    pub intern_table: HashMap<u64, Rc<str>>,
    pub expose_interns: bool,
}

#[allow(unused)]
impl VM {
    pub fn new() -> Self {
        Self {
            pos: 0,
            instructions: vec![],
            stack: Vec::with_capacity(100),
            call_stack: vec![CallFrame {
                scope_base: 0,
                return_addr: 0,
                upvalues: vec![],
            }],
            constants: Vec::with_capacity(100),
            globals: Self::initialize_globals(),
            locals: vec![rc!(RefCell::new(HashMap::new()))],
            libraries: Self::initialize_libs(),
            iterators: vec![],
            intern_table: HashMap::new(),
            expose_interns: true,
        }
    }

    pub fn initialize_globals() -> HashMap<u64, (Value, bool)> {
        let mut globals = HashMap::new();
        globals.insert(hash_u64!("Std"), (load_standard_namespace(), true));

        // Global Builtins
        globals.insert(
            hash_u64!("println"),
            (lib_function!("io", "write_line"), false),
        );
        globals.insert(hash_u64!("print"), (lib_function!("io", "write"), false));
        globals.insert(
            hash_u64!("typeof"),
            (lib_function!("type", "typeof"), false),
        );
        globals.insert(
            hash_u64!("number"),
            (lib_function!("type", "number"), false),
        );
        globals.insert(hash_u64!("char"), (lib_function!("type", "char"), false));
        globals.insert(hash_u64!("bool"), (lib_function!("type", "bool"), false));
        globals.insert(
            hash_u64!("string"),
            (lib_function!("type", "string"), false),
        );

        return globals;
    }

    pub fn initialize_libs() -> HashMap<u64, Box<dyn Library>> {
        let mut libs: HashMap<_, Box<dyn Library>> = HashMap::new();

        // types
        libs.insert(hash_u64!("type"), Box::new(TypeLib));
        libs.insert(hash_u64!("string"), Box::new(StringLib));
        libs.insert(hash_u64!("list"), Box::new(ListLib));
        libs.insert(hash_u64!("tuple"), Box::new(TupleLib));
        libs.insert(hash_u64!("dict"), Box::new(DictLib));

        // namespaces
        libs.insert(hash_u64!("math"), Box::new(MathLib));
        libs.insert(hash_u64!("io"), Box::new(IOLib));

        libs
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
    pub fn pop_or_nil(&mut self) -> Value {
        return self.stack.pop().unwrap_or(Value::NIL);
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

    pub fn call_function(&mut self, f: TFunction, mut args_count: usize) {
        if let Some((library, method)) = f.handler {
            if let Some(this) = f.this {
                self.stack.push(*this);
                args_count += 1;
            }
            let mut args: Vec<_> = (0..args_count).map(|_| self.pop()).collect();

            if let Some(lib) = self.libraries.get(&library) {
                let value = lib.get_function(method)(self, args);
                self.stack.push(value);
            }
        } else {
            self.call_stack.push(CallFrame {
                scope_base: self.locals.len(),
                return_addr: self.pos,
                upvalues: f.upvalues,
            });
            self.pos = f.entry;
        }
    }

    pub fn lookup_intern(&self, id: u64) -> Rc<str> {
        if !self.expose_interns {
            return rc!("<unknown>");
        }
        self.intern_table
            .get(&id)
            .unwrap_or(&rc!("<unknown>"))
            .clone()
    }

    pub fn print_instructions(&self) {
        let mut depth: i32 = 0;

        for (i, v) in self.instructions.iter().enumerate() {
            if let Inst::POP_SCOPE = v {
                depth -= 1;
            }

            let indent = format!(
                "{}{}",
                if depth < 0 { RED } else { DIM_BLACK },
                "|  ".repeat(depth.abs() as usize)
            );

            if let Inst::COMMENT(x) = v {
                println!(
                    "{BLACK}\t {}--- {x} ---{RESET}",
                    format!(
                        "{}{}",
                        if depth < 0 { RED } else { DIM_BLACK },
                        "|  ".repeat(depth.abs() as usize)
                    ),
                );
                continue;
            }

            // Resolve u64 instructions to human-readable display strings
            let display = match v {
                Inst::LOAD(id) => Some(format!("LOAD({})", self.lookup_intern(*id))),
                Inst::LOAD_LOCAL { id, depth } => Some(format!(
                    "LOAD_LOCAL({}, depth: {})",
                    self.lookup_intern(*id),
                    depth
                )),
                Inst::LOAD_GLOBAL(id) => Some(format!("LOAD_GLOBAL({})", self.lookup_intern(*id))),
                Inst::STORE_LOCAL { id, depth } => Some(format!(
                    "STORE_LOCAL({}, depth: {})",
                    self.lookup_intern(*id),
                    depth
                )),
                Inst::STORE_LOCAL_CONST { id, depth } => Some(format!(
                    "STORE_LOCAL_CONST({}, depth: {})",
                    self.lookup_intern(*id),
                    depth
                )),
                Inst::STORE_GLOBAL(id) => {
                    Some(format!("STORE_GLOBAL({})", self.lookup_intern(*id)))
                }
                Inst::SET_VAR(id) => Some(format!("SET_VAR({})", self.lookup_intern(*id))),
                Inst::LOAD_UPVALUE { id, scope_idx } => Some(format!(
                    "LOAD_UPVALUE({}, depth: {})",
                    self.lookup_intern(*id),
                    depth
                )),
                Inst::MAKE_CLOSURE { entry, captures } => Some(format!(
                    "MAKE_CLOSURE(entry: {}, captures: {:?})",
                    entry, captures
                )),
                _ => None,
            };

            let s = match &display {
                Some(d) => d.clone(),
                None => format!("{:?}", v),
            };

            let mut parts = s.splitn(2, '(');
            let opcode = parts.next().unwrap();
            let rest = parts.next().map_or("", |r| r);

            let color = if matches!(v, Inst::EXIT | Inst::RETURN) {
                RED
            } else if matches!(
                v,
                Inst::LIST(_)
                    | Inst::TUPLE(_)
                    | Inst::DICT(_)
                    | Inst::ENUM(..)
                    | Inst::STRUCT(..)
                    | Inst::RANGE
            ) {
                GREEN
            } else if matches!(v, Inst::NOP) {
                BLACK
            } else {
                ORANGE
            };

            print!("{MAGENTA}{i:>2}{RESET}   {BLACK}{indent}");

            if rest.is_empty() {
                print!("{color}{opcode}{RESET}");
            } else {
                print!(
                    "{color}{opcode}{RESET}({BLUE}{}{RESET})",
                    &rest[0..rest.len() - 1]
                );
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
                println!();
            }
        }
    }
}

// BYTECODE
impl VM {
    pub fn read_bytecode_file(&mut self, path: &str) {
        let bytecode_file = std::fs::read(path).unwrap();

        let config = bincode::config::standard().with_variable_int_encoding();
        let decoded: (Chunk, _) = bincode::decode_from_slice(&bytecode_file, config).unwrap();

        self.constants = decoded.0.constants;
        self.instructions = decoded.0.instructions;
    }

    pub fn write_bytecode_file(&mut self, path: &str) {
        let chunk = Chunk::new(self.constants.clone(), self.instructions.clone());
        let config = bincode::config::standard().with_variable_int_encoding();
        let encoded = bincode::encode_to_vec(chunk, config).unwrap();

        std::fs::write(path, encoded).unwrap();
    }
}

// RUNNING
impl VM {
    pub fn pre_run_pass(&mut self) {
        self.fold_constants();
    }

    pub fn fold_constants(&mut self) {
        for inst in &mut self.instructions {
            if let Inst::LOAD_CONST(idx) = inst {
                *inst = Inst::PUSH(self.constants[*idx].clone());
            }
        }
    }

    pub fn run(&mut self, debug: bool, stop_at_return: bool) {
        let instructions = std::mem::take(&mut self.instructions);

        while self.pos < instructions.len() {
            if debug {
                println!("{BLACK}{} ...{RESET}", self.pos);
            }
            let current = &instructions[self.pos];

            match current {
                Inst::EXIT => return,
                Inst::NOP => {}
                Inst::COMMENT(_) => {}
                Inst::PRINT => println!("{}", self.pop().to_string(false)),
                Inst::TO_STRING => {
                    let s = self.pop();
                    self.stack.push(Value::string(s))
                }
                Inst::POP => {
                    self.pop();
                }
                Inst::TRY_POP => {
                    self.stack.pop();
                }
                Inst::DEFAULT => {
                    let default_value = self.pop();
                    let given_value = self.pop_or_nil();

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
                Inst::ENUM(name, name_vec) => {
                    let map = name_vec
                        .iter()
                        .rev()
                        .map(|x| (x.clone(), self.pop()))
                        .collect::<HashMap<_, _>>();

                    self.stack
                        .push(Value::Enum(TEnum::new(name.clone(), rc!(map))));
                }
                Inst::STRUCT(field_names) => {
                    let base_value = self.pop();
                    let base = if let Value::StructDef(base) = base_value {
                        base
                    } else {
                        panic!("Can only intialize struct definitions")
                    };

                    let values = rc!(RefCell::new(
                        field_names
                            .iter()
                            .map(|x| (x.clone(), self.pop()))
                            .collect::<HashMap<_, _>>()
                    ));

                    for (name, value) in values.borrow().iter() {
                        if let Some(v_type) = base.fields.get(&*name) {
                            if !value.type_matches(v_type) {
                                panic!(
                                    "Field '{name}' expects type `{v_type}`, got `{}`.",
                                    value.get_type()
                                )
                            }
                        } else {
                            panic!(
                                "Tried setting unknown field on struct of base {}",
                                base.name
                            );
                        }
                    }

                    self.stack.push(Value::Struct(TStruct::new(base, values)));
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
                        self.stack.push(Value::String(TString::new(format!(
                            "{}{}",
                            a.to_string(),
                            b.to_string()
                        ))));
                    } else if let (Value::String(a), Value::Char(b)) = (a, b) {
                        self.stack.push(Value::String(TString::new(format!(
                            "{}{}",
                            a.to_string(),
                            b
                        ))));
                    } else if let (Value::Char(a), Value::String(b)) = (a, b) {
                        self.stack.push(Value::String(TString::new(format!(
                            "{}{}",
                            a,
                            b.to_string()
                        ))));
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
                        self.stack
                            .push(Value::String(TString::new(a.0.repeat(*b as usize))));
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
                        (Value::NIL, Value::NIL) => true,
                        (Value::Number(x), Value::Number(y)) => x == y,
                        (Value::Bool(x), Value::Bool(y)) => x == y,
                        (Value::Char(x), Value::Char(y)) => x == y,

                        (Value::String(x), Value::String(y)) => {
                            Rc::ptr_eq(&x.0, &y.0) || *x.0 == *y.0
                        }

                        (Value::Char(x), Value::String(y)) => x.to_string() == *y.0,
                        (Value::String(y), Value::Char(x)) => x.to_string() == *y.0,

                        (a, b) => a == b,
                    };

                    self.stack.push(Value::Bool(result));
                }
                Inst::NEQ => {
                    let result = match self.pop_two() {
                        (Value::NIL, Value::NIL) => true,
                        (Value::Number(x), Value::Number(y)) => x != y,
                        (Value::Bool(x), Value::Bool(y)) => x != y,
                        (Value::Char(x), Value::Char(y)) => x != y,

                        (Value::String(x), Value::String(y)) => {
                            !Rc::ptr_eq(&x.0, &y.0) || *x.0 != *y.0
                        }

                        (Value::Char(x), Value::String(y)) => x.to_string() == *y.0,
                        (Value::String(y), Value::Char(x)) => x.to_string() == *y.0,

                        (a, b) => a != b,
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

                Inst::LOAD_CONST(id) => self.stack.push(self.constants[*id].clone()),
                Inst::STORE_GLOBAL(id) => {
                    let id = *id;
                    let value = self.pop();
                    self.globals.insert(id, (value, false));
                }
                Inst::LOAD_GLOBAL(id) => {
                    self.stack.push(
                        self.globals
                            .get(id)
                            .expect(&format!(
                                "Global `{}` doesn't exist.",
                                self.lookup_intern(*id)
                            ))
                            .0
                            .clone(),
                    );
                }

                Inst::PUSH_SCOPE => self.locals.push(rc!(RefCell::new(HashMap::new()))),
                Inst::POP_SCOPE => {
                    self.locals.pop();
                }
                Inst::STORE_LOCAL { id, depth } => {
                    if let Some(current_frame) = self.call_stack.last() {
                        let id = *id;
                        let depth = current_frame.scope_base + *depth;
                        let value = self.pop();
                        self.locals[depth].borrow_mut().insert(id, (value, false));
                    } else {
                        panic!("Too little CallFrames in call_stack (LOAD_LOCAL)")
                    }
                }
                Inst::LOAD_LOCAL { id, depth } => {
                    if let Some(current_frame) = self.call_stack.last() {
                        if let Some((val, _)) = self.locals[current_frame.scope_base + *depth]
                            .borrow()
                            .get(id)
                        {
                            self.stack.push(val.clone());
                        } else {
                            panic!(
                                "Unknown local variable at depth {}: {}",
                                current_frame.scope_base + *depth,
                                self.lookup_intern(*id)
                            );
                        }
                    } else {
                        panic!("Too little CallFrames in call_stack (LOAD_LOCAL)")
                    }
                }
                Inst::STORE_LOCAL_CONST { id, depth } => {
                    if let Some(current_frame) = self.call_stack.last() {
                        let id = *id;
                        let depth = current_frame.scope_base + *depth;
                        let value = self.pop();
                        self.locals[depth].borrow_mut().insert(id, (value, true));
                    } else {
                        panic!("Too little CallFrames in call_stack (LOAD_LOCAL)")
                    }
                }

                Inst::LOAD(name) => {
                    let mut found = None;

                    for scope in self.locals.iter().rev() {
                        if let Some(val) = scope.borrow().get(name) {
                            found = Some(val.clone());
                            break;
                        }
                    }

                    if let Some((val, _)) = found {
                        self.stack.push(val);
                    } else if let Some((val, _)) = self.globals.get(name) {
                        self.stack.push(val.clone());
                    } else {
                        panic!(
                            "Unknown local/global variable: {}",
                            self.lookup_intern(*name)
                        );
                    }
                }
                Inst::SET_VAR(name) => {
                    let mut found_idx = None;

                    for i in (0..self.locals.len()).rev() {
                        if let Some((_, is_const)) = self.locals[i].borrow().get(name) {
                            if *is_const {
                                panic!("Cannot set a constant `{name}`");
                            } else {
                                found_idx = Some(i);
                            }
                            break;
                        }
                    }
                    if let Some(scope) = found_idx {
                        let value = self.pop();
                        if let Some(slot) = self.locals[scope].borrow_mut().get_mut(&name) {
                            slot.0 = value;
                        }
                    } else if let Some((_, is_const)) = self.globals.get(name) {
                        if *is_const {
                            panic!("Cannot set a global constant `{name}`");
                        } else {
                            let value = self.pop();
                            if let Some(slot) = self.globals.get_mut(&name) {
                                slot.0 = value;
                            }
                        }
                    }
                }

                Inst::MAKE_CLOSURE { entry, captures } => {
                    let upvalues = captures
                        .iter()
                        .map(|&i| Rc::clone(&self.locals[i]))
                        .collect();

                    self.stack.push(Value::Function(TFunction {
                        entry: *entry,
                        upvalues,
                        handler: None,
                        this: None,
                    }));
                }
                Inst::LOAD_UPVALUE { scope_idx, id } => {
                    if let Some(frame) = self.call_stack.last() {
                        let scope = &frame.upvalues[*scope_idx];
                        if let Some((val, _)) = scope.borrow().get(id) {
                            self.stack.push(val.clone());
                        } else {
                            panic!("Unknown upvalue: {}", self.lookup_intern(*id));
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
                Inst::JUMP_IF_TRUE(idx) => {
                    let idx = *idx;
                    if self.pop().is_truthy() {
                        self.pos = idx;
                        continue;
                    }
                }
                Inst::JUMP_IF_NOT_NIL(idx) => {
                    let idx = *idx;
                    if self.pop() != Value::NIL {
                        self.pos = idx;
                        continue;
                    }
                }

                Inst::CALL(args) => {
                    let arg_count = *args;
                    let func = self.pop();

                    if let Value::Function(f) = func {
                        let should_skip = f.handler.is_none();
                        self.call_function(f, arg_count);
                        if should_skip {
                            continue;
                        }
                    } else {
                        panic!("({}) Tried calling non-function: {func:?}", self.pos)
                    }
                }
                Inst::RETURN => {
                    if let Some(frame) = self.call_stack.last() {
                        self.pos = *&frame.return_addr;
                        self.locals.truncate(frame.scope_base);
                        self.call_stack.pop();
                    } else {
                        self.locals.pop();
                        break;
                    }
                    if stop_at_return {
                        self.locals.pop();
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

                        Value::Namespace(x) => {
                            let value = x.borrow().get_member(self, &member);
                            self.stack.push(value);
                        }

                        Value::Enum(x) => {
                            let value = x.get_member(self, &member);
                            self.stack.push(value);
                        }

                        Value::Struct(x) => {
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
                        Value::List(mut x) => {
                            x.set_member(&member, value);
                        }

                        Value::Tuple(mut x) => {
                            x.set_member(&member, value);
                        }

                        Value::Dict(mut x) => {
                            x.set_member(&member, value);
                        }

                        Value::Namespace(x) => {
                            x.borrow_mut().set_member(&member, value);
                        }

                        Value::Struct(mut x) => {
                            x.set_member(&member, value);
                        }

                        _ => panic!(
                            "Cannot set property `{member:?}` on `{}`",
                            target.to_string(false)
                        ),
                    }
                }

                Inst::GET_ITER => {
                    let value = self.pop();

                    if let Value::String(s) = &value {
                        let chars: Vec<Value> = s.0.chars().map(|c| Value::Char(c)).collect();
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

                Inst::MATCH => {
                    let result = match self.pop_two() {
                        (Value::NIL, Value::NIL) => true,
                        (Value::Number(x), Value::Number(y)) => x == y,
                        (Value::Bool(x), Value::Bool(y)) => x == y,
                        (Value::Char(x), Value::Char(y)) => x == y,

                        (Value::String(x), Value::String(y)) => {
                            Rc::ptr_eq(&x.0, &y.0) || *x.0 == *y.0
                        }

                        (Value::Char(x), Value::String(y)) => x.to_string() == *y.0,
                        (Value::String(y), Value::Char(x)) => x.to_string() == *y.0,

                        (a, b) => a == b,
                    };

                    self.stack.push(Value::Bool(result));
                }

                Inst::CONCAT_STR(n) => {
                    let values = (0..*n)
                        .map(|_| self.pop().to_string(false))
                        .collect::<String>();
                    self.stack.push(Value::String(TString::new(values)))
                }

                _ => panic!("Unimplemented instruction: {current:?}"),
            }

            self.advance();
        }
    }
}
