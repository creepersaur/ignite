use bincode::{Decode, Encode};

#[derive(Debug, Clone, Encode, Decode, Copy, PartialEq)]
pub enum NativeFunction {
    Println = 0,
    Print = 1,
}

impl NativeFunction {
    pub fn is_native(name: &str) -> Option<NativeFunction> {
        match name {
            "println" => Some(NativeFunction::Println),
            "print" => Some(NativeFunction::Print),

            _ => None,
        }
    }

    pub fn is_void(&self) -> bool {
        matches!(self, NativeFunction::Print | NativeFunction::Println)
    }
}
