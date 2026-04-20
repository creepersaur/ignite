use bincode::{Decode, Encode};

use crate::virtual_machine::value::Value;

#[allow(unused, non_camel_case_types)]
#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum Inst {
    COMMENT(String), // ✅
    NOP,             // ✅
    EXIT,            // ✅
    PRINT,           // ✅
    DEFAULT,         // ✅
    DEFAULT_NIL,     // ✅
    PUSH(Value),     // ✅
    DUP,             // ✅
    RANGE,           // ✅
    POP,             // ✅
    TRY_POP,         // ✅

    // Collections
    LIST(usize),  // ✅
    TUPLE(usize), // ✅
    DICT(usize),  // ✅
    ENUM(String, Vec<Value>),  // ✅

    PATCH_ME(String), // ✅

    ADD, // ✅
    SUB, // ✅
    MUL, // ✅
    DIV, // ✅
    POW, // ✅
    MOD, // ✅

    NEG, // ✅
    POS, // ✅

    GT,  // ✅
    LT,  // ✅
    GE,  // ✅
    LE,  // ✅
    EQ,  // ✅
    NEQ, // ✅
    AND, // ✅
    OR,  // ✅
    NOT, // ✅

    LOAD_CONST(usize), // ✅
    LOAD_GLOBAL(u64),  // ✅
    STORE_GLOBAL(u64), // ✅
    SET_VAR(u64),      // ✅

    PUSH_SCOPE,                                  // ✅
    POP_SCOPE,                                   // ✅
    LOAD_LOCAL { id: u64, depth: usize },        // ✅
    STORE_LOCAL { id: u64, depth: usize },       // ✅
    STORE_LOCAL_CONST { id: u64, depth: usize }, // ✅

    // Load from local or global
    LOAD(u64), // ✅

    JUMP(usize),            // ✅
    JUMP_IF_FALSE(usize),   // ✅
    JUMP_IF_TRUE(usize),    // ✅
    JUMP_IF_NOT_NIL(usize), // ✅

    // Get/Set property (member access)
    GET_PROP, // ✅
    SET_PROP, // ✅

    CALL(usize),      // ✅
    CALL_VOID(usize), // ✅
    RETURN,           // ✅

    // Get iterator (for loop)
    GET_ITER,        // ✅
    FOR_ITER(usize), // ✅

    // Match statement
    MATCH,

    // FString concatenation
    CONCAT_STR(usize),
}
