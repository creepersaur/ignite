use crate::{
    compiler::native_functions::NativeFunction,
    virtual_machine::{libs::types::TypeValue, value::Value},
};
use bincode::{Decode, Encode};
use std::rc::Rc;

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct ClassLayout {
    pub field_names: Vec<(u64, bool)>,
    pub method_names: Vec<u64>,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct ClosureLayout {
    pub entry: u32,
    pub captures: Vec<u32>,
}

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

    PUSH(Box<Value>),
    PUSH_TYPE(TypeValue),
    PUSH_NIL,
    PUSH_CHAR(char),
    PUSH_TRUE,
    PUSH_FALSE,
    PUSH_0,
    PUSH_1,

    DUP,
    DUP_N(u16),
    SWAP,
    ROT3,
    POP,
    TRY_POP,

    // Modules
    EXPORT(u64),
    IMPORT(Rc<str>),

    // Collections
    RANGE_INCLUSIVE,
    RANGE_EXCLUSIVE,
    LIST(u16),
    TUPLE(u16),
    DICT(u16),
    ENUM(Box<Vec<Value>>),
    STRUCT(Box<Vec<u64>>),
    MAKE_CLASS(Box<ClassLayout>),
    MAKE_CLASS_CONSTRUCTOR(Box<ClassLayout>),
    INIT_CLASS(u16),

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

    LOAD_CONST(u32),
    LOAD_GLOBAL(u64),
    STORE_GLOBAL(u64),
    STORE_GLOBAL_CONST(u64),

    SET_GLOBAL(u64),
    SET_LOCAL { id: u64, scope_idx: u16 },
    SET_UPVALUE { id: u64, scope_idx: u16 },

    PUSH_SCOPE,
    POP_SCOPE,
    LOAD_LOCAL { id: u64, depth: u16 },
    STORE_LOCAL { id: u64, depth: u16 },
    STORE_LOCAL_CONST { id: u64, depth: u16 },

    // UpValues
    MAKE_CLOSURE(Box<ClosureLayout>),
    LOAD_UPVALUE { id: u64, scope_idx: u16 },

    // Load from local or global
    LOAD(u64),

    JUMP(u32),
    JUMP_IF_FALSE(u32),
    JUMP_IF_TRUE(u32),
    JUMP_IF_NOT_NIL(u32),

    // Get/Set property (member access)
    GET_PROP,
    SET_PROP,
    GET_PROP_BY_ID(u64),
    SET_PROP_BY_ID(u64),

    FAST_CALL(NativeFunction, u16),
    FAST_CALL_VOID(NativeFunction, u16),
    CALL(u16),
    CALL_VOID(u16),
    RETURN,

    // Get iterator (for loop)
    GET_ITER,
    FOR_ITER(u32),

    // Match statement
    MATCH,

    // FString concatenation
    CONCAT_STR(u16),
}
