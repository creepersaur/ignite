use crate::{
    compiler::native_functions::NativeFunction,
    virtual_machine::{
        chunk::Chunk,
        inst::Inst,
        libs::{
            lib::Library,
            namespaces::{fs_lib::FSLib, io_lib::IOLib, math_lib::MathLib},
            type_lib::TypeLib,
            types::{
                TypeValue, dict_lib::DictLib, list_lib::ListLib, string_lib::StringLib,
                tuple_lib::TupleLib,
            },
        },
        namespaces::standard_namespace::load_standard_namespace,
        types::{
            classes::{class::TClass, class_object::TClassObject},
            dict::TDict,
            r#enum::TEnum,
            function::TFunction,
            list::TList,
            string::TString,
            r#struct::TStruct,
        },
        value::Value,
    },
};
use core::panic;
use lz4_flex::frame::{FrameDecoder, FrameEncoder};
use simply_colored::*;
use std::{cell::RefCell, collections::HashMap, io::Read, rc::Rc};

const ORANGE: &str = "\x1b[38;2;255;150;60m";
const BLUE: &str = "\x1b[38;2;115;165;255m"; // #73a5ff
const PURPLE: &str = "\x1b[38;2;197;156;249m"; // #c59cf9
const GREEN: &str = "\x1b[38;2;62;198;140m"; // #3ec68c

pub struct CallFrame {
    scope_base: usize,
    return_addr: usize,
    upvalues: Vec<Rc<RefCell<HashMap<u64, (Value, bool)>>>>,
}

pub struct VM {
    pub pos: usize,
    pub instructions: Rc<RefCell<Vec<Inst>>>,
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
            instructions: rc!(RefCell::new(vec![])),
            stack: Vec::with_capacity(600),
            call_stack: vec![CallFrame {
                scope_base: 0,
                return_addr: 0,
                upvalues: Vec::with_capacity(20),
            }],
            constants: Vec::with_capacity(150),
            globals: Self::initialize_globals(),
            locals: vec![rc!(RefCell::new(HashMap::new()))],
            libraries: Self::initialize_libs(),
            iterators: Vec::with_capacity(30),
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
            (lib_function!("IO", "write_line"), false),
        );
        globals.insert(hash_u64!("print"), (lib_function!("IO", "write"), false));
        globals.insert(
            hash_u64!("typeof"),
            (lib_function!("type", "typeof"), false),
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
        libs.insert(hash_u64!("Math"), Box::new(MathLib));
        libs.insert(hash_u64!("IO"), Box::new(IOLib));
        libs.insert(hash_u64!("FS"), Box::new(FSLib));

        libs
    }

    pub fn advance(&mut self) {
        self.pos += 1;
    }

    // pub fn push_inst(&mut self, instruction: Inst) {
    //     self.instructions.push(instruction);
    // }

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

    pub fn call_function(&mut self, f: TFunction, mut args_count: u16) {
        if let Some(target) = f.target {
            self.stack.push(target.as_ref().clone());
            args_count += 1;
        }

        if let Some((library, method)) = f.handler {
            if let Some(this) = f.this {
                self.stack.push(*this);
                args_count += 1;
            }
            let stack_start = self.stack.len() - args_count as usize;
            let mut args = self.stack.split_off(stack_start);
            args.reverse();

            if let Some(lib) = self.libraries.get(&library) {
                let value = lib.get_function(method)(self, args);
                self.stack.push(value);
            } else {
                panic!(
                    "Library not found for handler key: {} (method: {})",
                    library,
                    self.lookup_intern(method)
                );
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

    pub fn fast_call(&mut self, func: NativeFunction, args_count: u16) -> Value {
        let stack_len = self.stack.len();
        let stack_start = stack_len - args_count as usize;
        let args = &mut self.stack[stack_start..];
        if args_count > 1 {
            args.reverse();
        }

        let value = match func {
            NativeFunction::Print => IOLib::write_fast(args),
            NativeFunction::Println => IOLib::write_line_fast(args),

            _ => panic!("Unknown fast_call function: {func:?}"),
        };

        self.stack.truncate(stack_len - args_count as usize);

        value
    }

    pub fn lookup_intern(&self, id: u64) -> Rc<str> {
        if !self.expose_interns {
            return rc_str!("<unknown>");
        }
        self.intern_table
            .get(&id)
            .unwrap_or(&rc_str!("<unknown>"))
            .clone()
    }

    pub fn print_instructions(&self) {
        let mut depth: i32 = 0;

        let mut display_func = move |v: &Inst| {
            let display = match v {
                Inst::PUSH_SCOPE => {
                    depth += 1;
                    Some(format!("PUSH_SCOPE  +(depth: ({depth}))"))
                }
                Inst::POP_SCOPE => {
                    depth -= 1;
                    Some(format!("POP_SCOPE   -(depth: ({depth}))"))
                }
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
                Inst::STORE_GLOBAL_CONST(id) => {
                    Some(format!("STORE_GLOBAL_CONST({})", self.lookup_intern(*id)))
                }
                Inst::SET_GLOBAL(id) => Some(format!("SET_GLOBAL({})", self.lookup_intern(*id))),
                Inst::SET_LOCAL { id, scope_idx } => Some(format!(
                    "SET_LOCAL({}, depth: {scope_idx})",
                    self.lookup_intern(*id)
                )),
                Inst::SET_UPVALUE { id, scope_idx } => Some(format!(
                    "SET_UPVALUE({}, depth: {scope_idx})",
                    self.lookup_intern(*id)
                )),
                Inst::LOAD_UPVALUE { id, scope_idx } => Some(format!(
                    "LOAD_UPVALUE({}, depth: {})",
                    self.lookup_intern(*id),
                    depth
                )),
                Inst::MAKE_CLOSURE { entry, captures } => Some(format!(
                    "MAKE_CLOSURE(entry: {}, captures: {:?})",
                    entry, captures
                )),
                Inst::GET_PROP_BY_ID(id) => {
                    Some(format!("GET_PROP_BY_ID({})", self.lookup_intern(*id)))
                }
                Inst::SET_PROP_BY_ID(id) => {
                    Some(format!("SET_PROP_BY_ID({})", self.lookup_intern(*id)))
                }
                Inst::STRUCT(fields) => Some(format!(
                    "STRUCT({:?})",
                    fields
                        .iter()
                        .map(|x| self.lookup_intern(*x))
                        .collect::<Vec<_>>()
                )),
                Inst::MAKE_CLASS {
                    name,
                    has_constructor,
                    ..
                } => Some(format!(
                    "MAKE_CLASS(name: {}{})",
                    name,
                    if *has_constructor {
                        ", constructor"
                    } else {
                        ""
                    }
                )),
                _ => None,
            };

            display
        };

        let (max_opcode_width, max_operand_width) = self
            .instructions
            .borrow()
            .iter()
            .map(|v| {
                let display = display_func(v).unwrap_or(format!("{v:?}"));
                let mut parts = display.splitn(2, '(');

                let opcode_width = parts.next().unwrap().len();
                let operand_width = parts.next().map(|r| r.len().saturating_sub(1)).unwrap_or(0);

                (opcode_width, operand_width)
            })
            .fold((0, 0), |(max_op, max_arg), (op, arg)| {
                (max_op.max(op), max_arg.max(arg))
            });

        depth = 0;
        for (i, v) in self.instructions.borrow().iter().enumerate() {
            if let Inst::PUSH_SCOPE = v {
                depth += 1;
            } else if let Inst::POP_SCOPE = v {
                depth -= 1;
            }

            if let Inst::COMMENT(x) = v {
                println!("{BLACK}\t   --- {x} ---{RESET}",);
                continue;
            }

            let display = display_func(v);
            let s = match &display {
                Some(d) => d.clone(),
                None => format!("{:?}", v),
            };

            let mut parts = s.splitn(2, '(');

            let (opcode_width, operand_width) = (max_opcode_width, max_operand_width);
            let opcode = parts.next().unwrap();
            let opcode = format!("{opcode:<opcode_width$}");
            let opcode = opcode
                .replace("+", &format!("{GREEN}+{BLUE}"))
                .replace("-", &format!("{RED}-{BLUE}"));

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
                    | Inst::RANGE_EXCLUSIVE
                    | Inst::RANGE_INCLUSIVE
            ) {
                GREEN
            } else if matches!(v, Inst::NOP) {
                BLACK
            } else {
                BLUE
            };

            if depth < 0 {
                print!("{ORANGE}{i:>2}{RED} │ ");
            } else {
                print!("{ORANGE}{i:>2}{BLACK} │ ");
            }

            if rest.is_empty() {
                print!(
                    "{color}{opcode:<opcode_width$}{BLACK} │ {:<operand_width$} │ {RESET}",
                    ""
                );
            } else {
                let s = &rest[0..rest.len() - 1];

                if let Some((typename, rest)) = s.split_once('(') {
                    let contents = &rest[..rest.len() - 1];

                    print!("{color}{opcode:<opcode_width$}{BLACK} │ ",);

                    let plain_len = typename.len() + contents.len() + 2;
                    let padding = operand_width.saturating_sub(plain_len);

                    print!(
                        "{DIM_BLACK}{typename}{RESET}({GREEN}{contents}{RESET}){}{BLACK} │ {RESET}",
                        " ".repeat(padding)
                    );
                } else if let Some((typename, rest)) = s.split_once('[') {
                    let contents = &rest[..rest.len() - 1];

                    print!("{color}{opcode:<opcode_width$}{BLACK} │ ",);

                    let plain_len = typename.len() + contents.len() + 2;
                    let padding = operand_width.saturating_sub(plain_len);

                    print!(
                        "{DIM_BLACK}{typename}{RESET}[{GREEN}{contents}{RESET}]{}{BLACK} │ {RESET}",
                        " ".repeat(padding)
                    );
                } else {
                    print!(
                        "{color}{opcode:<opcode_width$}{BLACK} │ {PURPLE}{:<operand_width$}{BLACK} │ {RESET}",
                        &rest[0..rest.len() - 1]
                    );
                }
            }

            if let Inst::LOAD_CONST(x) = v {
                print!("{BLACK}{:?}{RESET}", self.constants[*x as usize]);
            }
            println!();
        }
    }
}

// BYTECODE
const MAGIC: &[u8; 4] = b"MYVM";

impl VM {
    pub fn read_bytecode_file(&mut self, path: &str) {
        let file = std::fs::read(path).unwrap();

        assert!(&file[0..4] == b"MYVM");

        let _version = file[4];
        let compressed = file[5] != 0;

        let payload = &file[6..];

        let decoded_bytes = if compressed {
            let mut decoder = FrameDecoder::new(payload);
            let mut out = vec![];
            decoder
                .read_to_end(&mut out)
                .expect("Could not decode compressed bytecode");
            out
        } else {
            payload.to_vec()
        };

        let config = bincode::config::standard().with_variable_int_encoding();
        let decoded: (Chunk, _) = bincode::decode_from_slice(&decoded_bytes, config).unwrap();

        self.constants = decoded.0.constants;
        self.instructions = rc!(RefCell::new(decoded.0.instructions));
    }

    pub fn write_bytecode_file(&mut self, path: &str, force_compress: bool) {
        let chunk = Chunk::new(
            self.constants.clone(),
            (*self.instructions.borrow()).clone(),
        );
        let config = bincode::config::standard().with_variable_int_encoding();
        let encoded = bincode::encode_to_vec(chunk, config).unwrap();
        let compressed = FrameEncoder::new(encoded.clone()).finish().unwrap();

        let use_compression = compressed.len() < encoded.len() || force_compress;
        let data = if use_compression { compressed } else { encoded };

        let mut file = vec![];
        file.extend_from_slice(MAGIC);
        file.push(1);
        file.push(use_compression as u8);
        file.extend(data);

        std::fs::write(path, file).unwrap();
    }
}

// RUNNING
impl VM {
    pub fn pre_run_pass(&mut self) {
        self.fold_constants();
    }

    pub fn fold_constants(&mut self) {
        for inst in &mut self.instructions.borrow_mut().iter_mut() {
            if let Inst::LOAD_CONST(idx) = inst {
                *inst = Inst::PUSH(self.constants[*idx as usize].clone());
            }
        }
    }

    #[inline(always)]
    pub fn reset(&mut self, hard: bool) {
        self.stack.clear();
        self.iterators.clear();
        self.call_stack.truncate(1);
        self.locals.truncate(1);
        self.locals[0].borrow_mut().clear();
        if hard {
            self.globals = Self::initialize_globals();
        }

        self.pos = 0;
    }

    pub fn run(&mut self, debug: bool, stop_at_return: bool) {
        let instructions = self.instructions.clone();

        while self.pos < instructions.borrow().len() {
            if debug {
                println!("{BLACK}{} ...{RESET}", self.pos);
            }
            let current = &instructions.borrow()[self.pos];

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
                Inst::PUSH_TYPE(t) => self.stack.push(Value::Type(*t)),
                Inst::PUSH_NIL => self.stack.push(Value::NIL),
                Inst::PUSH_TRUE => self.stack.push(Value::Bool(true)),
                Inst::PUSH_FALSE => self.stack.push(Value::Bool(false)),
                Inst::PUSH_0 => self.stack.push(Value::Number(0.0)),
                Inst::PUSH_1 => self.stack.push(Value::Number(1.0)),

                Inst::DUP => self.stack.push(
                    self.stack
                        .last()
                        .expect("Cannot DUP, stack underflow.")
                        .clone(),
                ),
                Inst::DUP_N(n) => {
                    let value = self.pop();
                    self.stack.push(value.clone());

                    self.stack.reserve(*n as usize);
                    for _ in 0..*n {
                        self.stack.push(value.clone());
                    }
                }
                Inst::SWAP => {
                    let n = self.stack.len();
                    self.stack.swap(n - 1, n - 2);
                }
                Inst::ROT3 => {
                    let n = self.stack.len();
                    self.stack.swap(n - 2, n - 1);
                    self.stack.swap(n - 3, n - 2);
                }

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
                        .map(|x| (x.as_str().into(), self.pop()))
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
                            .map(|x| (*x, self.pop()))
                            .collect::<HashMap<_, _>>()
                    ));

                    for (name, value) in values.borrow().iter() {
                        if let Some((v_type, _)) = base.fields.get(name) {
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
                Inst::MAKE_CLASS {
                    name,
                    field_names,
                    method_names,
                    has_constructor,
                } => {
                    let mut functions_map = HashMap::new();
                    for method_name in method_names.iter().rev() {
                        let closure = self.pop();
                        functions_map.insert(*method_name, closure);
                    }

                    let mut values_map = HashMap::new();
                    for (field_name, is_const) in field_names.iter().rev() {
                        let default_val = self.pop();
                        values_map.insert(*field_name, (default_val, *is_const));
                    }

                    let constructor = if *has_constructor {
                        Some(Box::new(self.pop()))
                    } else {
                        None
                    };

                    self.stack.push(Value::Class(rc!(RefCell::new(TClass::new(
                        Rc::from(name.clone()),
                        rc!(RefCell::new(values_map)),
                        rc!(RefCell::new(functions_map)),
                        constructor,
                    )))));
                }
                Inst::INIT_CLASS(args) => {
                    let target = self.pop();

                    if let Value::Class(c) = target {
                        let obj = Value::ClassObject(TClassObject::new(c.clone()));

                        if let Some(constructor) = &c.borrow().constructor {
                            if let Value::Function(f) = constructor.as_ref().clone() {
                                self.stack.push(obj.clone());
                                self.call_function(*f, args + 1);
                                self.run(false, true);
                                self.pop();
                            }
                        }

                        self.stack.push(obj);
                    } else if let Value::StructDef(_) = target {
                        panic!(
                            "`new ...()` can only be used to construct classes.
Use braces `new ...{{}}` to initialize a struct. Got {}",
                            target.get_type()
                        )
                    } else {
                        panic!(
                            "`new ...()` can only be used to construct classes. Got {}",
                            target.get_type()
                        )
                    }
                }

                Inst::RANGE_INCLUSIVE => {
                    let step = self.pop();
                    let end = self.pop();
                    let start = self.pop();

                    self.stack.push(Value::Range {
                        start: Box::new(start),
                        end: Box::new(end),
                        step: Box::new(step),
                        inclusive: true,
                    });
                }
                Inst::RANGE_EXCLUSIVE => {
                    let step = self.pop();
                    let end = self.pop();
                    let start = self.pop();

                    self.stack.push(Value::Range {
                        start: Box::new(start),
                        end: Box::new(end),
                        step: Box::new(step),
                        inclusive: false,
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
                    let (a, b) = self.pop_two();
                    if let (Value::Number(a), Value::Number(b)) = (&a, &b) {
                        self.stack.push(Value::Bool(a < b));
                    } else {
                        panic!("LT expects numbers, got {a:?} and {b:?}");
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
                Inst::IS_INSTANCE_OF => {
                    let result = match self.pop_two() {
                        (Value::NIL, Value::NIL) => true,
                        (Value::Number(_), Value::Type(TypeValue::Number)) => true,

                        (Value::Class(x), Value::Class(y)) => x == y,
                        (Value::ClassObject(x), Value::Class(y)) => x.base == y,

                        (Value::StructDef(x), Value::StructDef(y)) => x == y,
                        (Value::Struct(x), Value::StructDef(y)) => x.base == y,

                        _ => false,
                    };

                    self.stack.push(Value::Bool(result));
                }

                Inst::LOAD_CONST(id) => self.stack.push(self.constants[*id as usize].clone()),
                Inst::STORE_GLOBAL(id) => {
                    let id = *id;
                    let value = self.pop();
                    self.globals.insert(id, (value, false));
                }
                Inst::STORE_GLOBAL_CONST(id) => {
                    let id = *id;
                    let value = self.pop();
                    self.globals.insert(id, (value, true));
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
                        let depth = current_frame.scope_base + *depth as usize;
                        let value = self.pop();
                        self.locals[depth].borrow_mut().insert(id, (value, false));
                    } else {
                        panic!("Too little CallFrames in call_stack (STORE_LOCAL)")
                    }
                }
                Inst::LOAD_LOCAL { id, depth } => {
                    if let Some(current_frame) = self.call_stack.last() {
                        if let Some((val, _)) = self.locals
                            [current_frame.scope_base + *depth as usize]
                            .borrow()
                            .get(id)
                        {
                            self.stack.push(val.clone());
                        } else {
                            panic!(
                                "Unknown local variable at depth {}: {}",
                                current_frame.scope_base + *depth as usize,
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
                        let depth = current_frame.scope_base + *depth as usize;
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
                Inst::SET_GLOBAL(id) => {
                    let new_value = self.pop();

                    if let Some((value, is_const)) = self.globals.get_mut(id) {
                        if *is_const {
                            panic!("Cannot set constant global `{}`", self.lookup_intern(*id))
                        }

                        *value = new_value;
                    } else {
                        panic!("Tried setting unknown global `{}`", self.lookup_intern(*id))
                    }
                }
                Inst::SET_LOCAL { id, scope_idx } => {
                    if let Some(current_frame) = self.call_stack.last() {
                        let depth = current_frame.scope_base + *scope_idx as usize;
                        let new_value = self.pop();

                        if let Some((value, is_const)) = self.locals[depth].borrow_mut().get_mut(id)
                        {
                            if *is_const {
                                panic!("Cannot set constant local `{}`", self.lookup_intern(*id))
                            }

                            *value = new_value;
                        } else {
                            panic!(
                                "Tried setting unknown local variable `{}`",
                                self.lookup_intern(*id)
                            )
                        }
                    } else {
                        panic!("Too little CallFrames in call_stack (SET_LOCAL)")
                    }
                }
                Inst::SET_UPVALUE { id, scope_idx } => {
                    let new_value = self.pop();

                    if let Some(current_frame) = self.call_stack.last() {
                        if let Some((value, is_const)) =
                            current_frame.upvalues[*scope_idx as usize].borrow_mut().get_mut(id)
                        {
                            if *is_const {
                                panic!("Cannot set constant local `{}`", self.lookup_intern(*id))
                            }

                            *value = new_value;
                        } else {
                            panic!(
                                "Tried setting unknown local variable `{}`",
                                self.lookup_intern(*id)
                            )
                        }
                    } else {
                        panic!("Too little CallFrames in call_stack (SET_LOCAL)")
                    }
                }

                Inst::MAKE_CLOSURE { entry, captures } => {
                    let upvalues = captures
                        .iter()
                        .map(|&i| Rc::clone(&self.locals[i as usize]))
                        .collect();

                    self.stack.push(Value::Function(Box::new(TFunction {
                        entry: *entry as usize,
                        upvalues,
                        handler: None,
                        this: None,
                        target: None,
                    })));
                }
                Inst::LOAD_UPVALUE { scope_idx, id } => {
                    if let Some(frame) = self.call_stack.last() {
                        let scope = &frame.upvalues[*scope_idx as usize];
                        if let Some((val, _)) = scope.borrow().get(id) {
                            self.stack.push(val.clone());
                        } else {
                            panic!("Unknown upvalue: {}", self.lookup_intern(*id));
                        }
                    }
                }

                Inst::JUMP(idx) => {
                    self.pos = *idx as usize;
                    continue;
                }
                Inst::JUMP_IF_FALSE(idx) => {
                    let idx = *idx;
                    if !self.pop().is_truthy() {
                        self.pos = idx as usize;
                        continue;
                    }
                }
                Inst::JUMP_IF_TRUE(idx) => {
                    let idx = *idx;
                    if self.pop().is_truthy() {
                        self.pos = idx as usize;
                        continue;
                    }
                }
                Inst::JUMP_IF_NOT_NIL(idx) => {
                    let idx = *idx;
                    if self.pop() != Value::NIL {
                        self.pos = idx as usize;
                        continue;
                    }
                }

                Inst::CALL(args) => {
                    let arg_count = *args;
                    let func = self.pop();

                    if let Value::Function(f) = func {
                        let should_skip = f.handler.is_none();
                        self.call_function(*f, arg_count);
                        if should_skip {
                            continue;
                        }
                    } else {
                        let args = (0..arg_count).map(|_| self.pop()).collect::<Vec<_>>();

                        match func {
                            Value::Type(TypeValue::Number) => {
                                let value = &args[0];

                                match value {
                                    Value::Number(x) => self.stack.push(Value::Number(*x)),

                                    Value::Bool(x) => {
                                        self.stack.push(Value::Number(if *x { 1.0 } else { 0.0 }))
                                    }

                                    Value::Char(x) => {
                                        self.stack.push(Value::Number(*x as u32 as f64))
                                    }

                                    Value::String(x) => {
                                        self.stack.push(Value::Number(x.0.parse::<f64>().expect(
                                            &format!("Could not convert `{}` to number", x.0),
                                        )))
                                    }

                                    _ => panic!("Cannot cast {value:?} to number"),
                                }
                            }

                            Value::Type(TypeValue::Char) => {
                                let value = &args[0];
                                match value {
                                    Value::Number(x) => self.stack.push(Value::Char(
                                        char::from_u32(*x as u32)
                                            .expect(&format!("Could not convert `{x}` to char")),
                                    )),

                                    Value::String(x) => {
                                        self.stack.push(Value::Char(x.0.chars().next().unwrap()))
                                    }

                                    Value::Char(x) => self.stack.push(Value::Char(*x)),

                                    _ => panic!("Cannot cast {value:?} to char"),
                                }
                            }

                            Value::Type(TypeValue::Bool) => {
                                let value = &args[0];
                                if let Value::Number(x) = value {
                                    if *x == 0.0 {
                                        self.stack.push(Value::Bool(false))
                                    }
                                }
                                self.stack.push(Value::Bool(value.is_truthy()))
                            }

                            _ => panic!("({}) Tried calling non-function: {func:?}", self.pos),
                        }
                    }
                }
                Inst::FAST_CALL(func, args) => {
                    let value = self.fast_call(*func, *args);
                    self.stack.push(value);
                }
                Inst::FAST_CALL_VOID(func, args) => {
                    self.fast_call(*func, *args);
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
                        break;
                    }
                }

                Inst::GET_PROP => {
                    let member = self.pop();
                    let target = self.pop();

                    let value = target.get_member(self, &member);
                    self.stack.push(value);
                }
                Inst::SET_PROP => {
                    let member = self.pop();
                    let mut target = self.pop();
                    let value = self.pop();

                    target.set_member(&member, value);
                }
                Inst::GET_PROP_BY_ID(id) => {
                    let target = self.pop();

                    let value = target.get_member_id(self, id);
                    self.stack.push(value);
                }
                Inst::SET_PROP_BY_ID(id) => {
                    let mut target = self.pop();
                    let value = self.pop();

                    target.set_member_id(self, id, value);
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
                            self.pos = *jump_end as usize;
                            continue;
                        }
                    } else if let Value::Tuple(list) = value {
                        if *idx < list.values.borrow().len() {
                            self.stack.push(list.values.borrow()[*idx].clone());
                            *idx += 1;
                        } else {
                            self.iterators.pop();
                            self.pos = *jump_end as usize;
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
                            self.pos = *jump_end as usize;
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
