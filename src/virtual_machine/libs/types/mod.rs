use bincode::{Decode, Encode};

pub mod dict_lib;
pub mod list_lib;
pub mod string_lib;
pub mod tuple_lib;

#[derive(Encode, Decode, Debug, Clone, Copy, PartialEq, PartialOrd, Hash)]
pub enum TypeValue {
    Number,
    String,
    Bool,
    Char,
    List,
    Dict,
    Tuple,
    Function,
}
