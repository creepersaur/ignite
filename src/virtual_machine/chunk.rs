use bincode::{Decode, Encode};

use crate::virtual_machine::{inst::Inst, value::Value};

#[derive(Encode, Decode)]
pub struct Chunk {
    pub constants: Vec<Value>,
    pub instructions: Vec<Inst>,
}

impl Chunk {
    pub fn new(constants: Vec<Value>, instructions: Vec<Inst>) -> Self {
        Self {
            constants,
            instructions,
        }
    }
}
