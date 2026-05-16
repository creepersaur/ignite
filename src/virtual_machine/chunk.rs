use crate::virtual_machine::{inst::Inst, value::Value};
use bincode::{Decode, Encode};

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
