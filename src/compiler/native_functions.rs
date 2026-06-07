use bincode::{Decode, Encode};

#[repr(u8)]
#[derive(Debug, Clone, Encode, Decode, Copy, PartialEq)]
pub enum NativeFunction {
    Println,
    Print,
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
