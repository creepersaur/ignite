use std::rc::Rc;

use bincode::{Decode, Encode};

use crate::virtual_machine::{libs::types::TypeValue, value::Value};

#[allow(unused, non_camel_case_types)]
#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum Inst {
    COMMENT(Rc<str>),
    NOP,
    EXIT,
    PRINT,
    TO_STRING,
    DEFAULT,
    DEFAULT_NIL,

	PUSH(Value),
	PUSH_TYPE(TypeValue),
	PUSH_NIL,
	PUSH_TRUE,
	PUSH_FALSE,

    DUP,
    SWAP,
    ROT3,
    POP,
    TRY_POP,

    // Collections
    RANGE,
    LIST(usize),
    TUPLE(usize),
    DICT(usize),
    ENUM(Rc<str>, Vec<Value>),
    STRUCT(Vec<Rc<str>>),
    MAKE_CLASS {
        name: Rc<str>,
        field_names: Vec<Rc<str>>,
        field_consts: Vec<bool>,
        method_names: Vec<Rc<str>>,
        has_constructor: bool,
    },
    INIT_CLASS(usize),

    PATCH_ME(Rc<str>),

    ADD,
    SUB,
    MUL,
    DIV,
    POW,
    MOD,

    NEG,
    POS,

    GT,
    LT,
    GE,
    LE,
    EQ,
    NEQ,
    AND,
    OR,
    NOT,
	IS_INSTANCE_OF,

    LOAD_CONST(usize),
    LOAD_GLOBAL(u64),
    STORE_GLOBAL(u64),
    STORE_GLOBAL_CONST(u64),
    SET_VAR(u64),

    PUSH_SCOPE,
    POP_SCOPE,
    LOAD_LOCAL {
        id: u64,
        depth: usize,
    },
    STORE_LOCAL {
        id: u64,
        depth: usize,
    },
    STORE_LOCAL_CONST {
        id: u64,
        depth: usize,
    },

    // UpValues
    MAKE_CLOSURE {
        entry: usize,
        captures: Vec<usize>,
    },
    LOAD_UPVALUE {
        id: u64,
        scope_idx: usize,
    },

    // Load from local or global
    LOAD(u64),

    JUMP(usize),
    JUMP_IF_FALSE(usize),
    JUMP_IF_TRUE(usize),
    JUMP_IF_NOT_NIL(usize),

    // Get/Set property (member access)
    GET_PROP,
    SET_PROP,

    CALL(usize),
    CALL_VOID(usize),
    RETURN,

    // Get iterator (for loop)
    GET_ITER,
    FOR_ITER(usize),

    // Match statement
    MATCH,

    // FString concatenation
    CONCAT_STR(usize),
}
