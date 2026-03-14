use bincode::{Decode, Encode};
use std::rc::Rc;

use crate::virtual_machine::value::Value;

#[allow(unused, non_camel_case_types)]
#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum Inst {
    EXIT,        // ✅
    NOP,         // ✅
    PRINT,       // ✅
    DEFAULT,     // ✅
    DEFAULT_NIL, // ✅
    PUSH(Value), // ✅
    DUP,         // ✅
    RANGE,       // ✅
    POP,         // ✅

	// Collections
    LIST(usize), // ✅
    TUPLE(usize), // ✅
    DICT(usize), // ✅

    PATCH_ME(String), // ✅

    ADD, // ✅
    SUB, // ✅
    MUL, // ✅
    DIV, // ✅
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

    LOAD_CONST(usize),        // ✅
    LOAD_GLOBAL(Rc<String>),  // ✅
    STORE_GLOBAL(Rc<String>), // ✅
    SET_VAR(Rc<String>),      // ✅

    PUSH_SCOPE,                    // ✅
    POP_SCOPE,                     // ✅
    LOAD_LOCAL(Rc<String>),        // ✅
    STORE_LOCAL(Rc<String>),       // ✅
    STORE_LOCAL_CONST(Rc<String>), // ✅

    // Load from local or global
    LOAD(Rc<String>), // ✅

    JUMP(usize),          // ✅
    JUMP_IF_FALSE(usize), // ✅

    // Get/Set property (member access)
    GET_PROP, // ✅
    SET_PROP, // ✅

    CALL,                            // ✅
    CALL_BUILTIN(Rc<String>, usize), // ✅
    RETURN,                          // ✅

    // Get iterator (for loop)
    GET_ITER,        // ✅
    FOR_ITER(usize), // ✅
}
