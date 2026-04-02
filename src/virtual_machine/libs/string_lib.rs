use crate::{
    get_args, hash_u64, rc, virtual_machine::{
        libs::lib::Library,
        types::{list::TList, string::TString},
        value::Value,
        vm::VM,
    }
};
use std::cell::RefCell;

pub const STRING_FUNCTIONS: [&str; 31] = [
    "len",
    "concat",
    "copy",
    "count",
    "reverse",
    "rep",
    "chars",
    "bytes",
    "split",
    "upper",
    "lower",
    "trim",
    "ltrim",
    "rtrim",
    "replace",
    "starts_with",
    "ends_with",
    "find",
    "title",
    "center",
    "ljust",
    "rjust",
    "is_upper",
    "is_lower",
    "is_numeric",
    "is_alphanumeric",
    "is_alpha",
    "is_ascii",
    "is_empty",
    "is_whitespace",
    "join",
];

pub struct StringLib;

impl StringLib {
    fn len(_vm: &mut VM, args: Vec<Value>) -> Value {
        let string = get_args!(args);

        if let Value::String(inner) = string {
            Value::Number(inner.0.len() as f64)
        } else {
            panic!("Can only use string.len on strings");
        }
    }

    fn join(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [string, list] = get_args!(args, 2);

        if let Value::List(inner) = list {
            if let Value::String(t) = string {
                Value::String(TString::new(
                    inner
                        .values
                        .borrow()
                        .iter()
                        .map(|x| x.to_string(false))
                        .collect::<Vec<_>>()
                        .join(&t.0),
                ))
            } else {
                panic!("Expected string for separator");
            }
        } else {
            panic!("Can only use string.concat on a list");
        }
    }

    fn concat(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [string, other] = get_args!(args, 2);

        if let Value::String(inner) = string {
            return Value::String(TString::new(format!(
                "{}{}",
                inner.0,
                other.to_string(false)
            )));
        } else {
            panic!("Can only use string.concat on strings");
        }
    }

    fn copy(_vm: &mut VM, args: Vec<Value>) -> Value {
        let string = get_args!(args);

        if let Value::String(inner) = string {
            Value::String(TString(std::rc::Rc::from(&*inner.0)))
        } else {
            panic!("Can only use string.copy on strings");
        }
    }

    fn count(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [string, item] = get_args!(args, 2);

        if let Value::String(inner) = string {
            let count = inner.0.matches(&*item.to_string(false)).count();
            Value::Number(count as f64)
        } else {
            panic!("Can only use string.count on strings");
        }
    }

    fn reverse(_vm: &mut VM, args: Vec<Value>) -> Value {
        let string = get_args!(args);

        if let Value::String(inner) = string {
            Value::String(TString::new(inner.0.chars().rev().collect::<String>()))
        } else {
            panic!("Can only use string.reverse on strings");
        }
    }

    fn rep(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [string, value] = get_args!(args, 2);

        if let Value::String(inner) = string {
            if let Value::Number(n) = value {
                return Value::String(TString::new(inner.0.repeat(n as usize)));
            } else {
                panic!("Can only string.repeat with a number")
            }
        } else {
            panic!("Can only use string.repeat on strings");
        }
    }

    fn bytes(_vm: &mut VM, args: Vec<Value>) -> Value {
        let string = get_args!(args);

        if let Value::String(inner) = string {
            return Value::List(TList::new(rc!(RefCell::new(
                inner
                    .0
                    .bytes()
                    .map(|x| Value::Number(x as f64))
                    .collect::<Vec<_>>()
            ))));
        } else {
            panic!("Can only use string.bytes on strings");
        }
    }

    fn chars(_vm: &mut VM, args: Vec<Value>) -> Value {
        let string = get_args!(args);

        if let Value::String(inner) = string {
            return Value::List(TList::new(rc!(RefCell::new(
                inner.0.chars().map(|x| Value::Char(x)).collect::<Vec<_>>()
            ))));
        } else {
            panic!("Can only use string.chars on strings");
        }
    }

    fn split(_vm: &mut VM, args: Vec<Value>) -> Value {
        let string = get_args!(args);
        let new_value = args.get(1).unwrap_or(&Value::NIL);

        if let Value::String(inner) = string {
            if let Value::String(value) = new_value {
                Value::List(TList::new(rc!(RefCell::new(
                    inner
                        .0
                        .split(&*value.0)
                        .map(|x: &str| Value::String(TString::from_str(x)))
                        .collect::<Vec<_>>()
                ))))
            } else if let Value::Char(c) = new_value {
                Value::List(TList::new(rc!(RefCell::new(
                    inner
                        .0
                        .split(*c)
                        .map(|x: &str| Value::String(TString::from_str(x)))
                        .collect::<Vec<_>>()
                ))))
            } else if Value::NIL == *new_value {
                Value::List(TList::new(rc!(RefCell::new(
                    inner
                        .0
                        .split(" ")
                        .map(|x: &str| Value::String(TString::from_str(x)))
                        .collect::<Vec<_>>()
                ))))
            } else {
                panic!("Can only split() a string value with a string separator")
            }
        } else {
            panic!("Can only use string.split on strings")
        }
    }

    fn upper(_vm: &mut VM, args: Vec<Value>) -> Value {
        let string = get_args!(args);

        if let Value::String(inner) = string {
            Value::String(TString::new(inner.0.to_uppercase()))
        } else {
            panic!("Can only use string.upper on strings");
        }
    }

    fn lower(_vm: &mut VM, args: Vec<Value>) -> Value {
        let string = get_args!(args);

        if let Value::String(inner) = string {
            Value::String(TString::new(inner.0.to_lowercase()))
        } else {
            panic!("Can only use string.lower on strings");
        }
    }

    // --- Trim ---

    fn trim(_vm: &mut VM, args: Vec<Value>) -> Value {
        let string = get_args!(args);

        if let Value::String(inner) = string {
            Value::String(TString::new(inner.0.trim().to_string()))
        } else {
            panic!("Can only use string.trim on strings");
        }
    }

    fn ltrim(_vm: &mut VM, args: Vec<Value>) -> Value {
        let string = get_args!(args);

        if let Value::String(inner) = string {
            Value::String(TString::new(inner.0.trim_start().to_string()))
        } else {
            panic!("Can only use string.ltrim on strings");
        }
    }

    fn rtrim(_vm: &mut VM, args: Vec<Value>) -> Value {
        let string = get_args!(args);

        if let Value::String(inner) = string {
            Value::String(TString::new(inner.0.trim_end().to_string()))
        } else {
            panic!("Can only use string.rtrim on strings");
        }
    }

    // --- Search & Replace ---

    /// replace(old, new) -> string
    fn replace(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [string, new_val, old_val] = get_args!(args, 3);

        if let Value::String(inner) = string {
            let old = old_val.to_string(false);
            let new = new_val.to_string(false);
            Value::String(TString::new(inner.0.replace(&*old, &new)))
        } else {
            panic!("Can only use string.replace on strings");
        }
    }

    /// find(sub) -> number (index) or nil if not found
    fn find(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [string, sub] = get_args!(args, 2);

        if let Value::String(inner) = string {
            let needle = sub.to_string(false);
            match inner.0.find(&*needle) {
                Some(idx) => Value::Number(idx as f64),
                None => Value::NIL,
            }
        } else {
            panic!("Can only use string.find on strings");
        }
    }

    /// starts_with(prefix) -> bool
    fn starts_with(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [string, prefix] = get_args!(args, 2);

        if let Value::String(inner) = string {
            let p = prefix.to_string(false);
            Value::Bool(inner.0.starts_with(&*p))
        } else {
            panic!("Can only use string.starts_with on strings");
        }
    }

    /// ends_with(suffix) -> bool
    fn ends_with(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [string, suffix] = get_args!(args, 2);

        if let Value::String(inner) = string {
            let s = suffix.to_string(false);
            Value::Bool(inner.0.ends_with(&*s))
        } else {
            panic!("Can only use string.ends_with on strings");
        }
    }

    // --- Case Conversion ---

    /// title() -> string  (Title Case Every Word)
    fn title(_vm: &mut VM, args: Vec<Value>) -> Value {
        let string = get_args!(args);

        if let Value::String(inner) = string {
            let titled = inner
                .0
                .split_whitespace()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => {
                            first.to_uppercase().collect::<String>()
                                + &chars.as_str().to_lowercase()
                        }
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            Value::String(TString::new(titled))
        } else {
            panic!("Can only use string.title on strings");
        }
    }

    // --- Padding / Alignment ---

    /// center(width) or center(width, fill_char) -> string
    fn center(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [string, fill_val, width_val] = get_args!(args, 3);

        if let Value::String(inner) = string {
            let width = if let Value::Number(n) = width_val {
                n as usize
            } else {
                panic!("string.center: width must be a number");
            };

            let fill = match fill_val {
                Value::Char(c) => c,
                Value::NIL => ' ',
                _ => panic!("string.center: fill must be a char or nil"),
            };

            let len = inner.0.chars().count();
            if len >= width {
                return Value::String(inner.clone());
            }
            let total_pad = width - len;
            let left_pad = total_pad / 2;
            let right_pad = total_pad - left_pad;
            let result = format!(
                "{}{}{}",
                fill.to_string().repeat(left_pad),
                inner.0,
                fill.to_string().repeat(right_pad)
            );
            Value::String(TString::new(result))
        } else {
            panic!("Can only use string.center on strings");
        }
    }

    /// ljust(width) or ljust(width, fill_char) -> string
    fn ljust(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [string, fill_val, width_val] = get_args!(args, 3);

        if let Value::String(inner) = string {
            let width = if let Value::Number(n) = width_val {
                n as usize
            } else {
                panic!("string.ljust: width must be a number");
            };

            let fill = match fill_val {
                Value::Char(c) => c,
                Value::NIL => ' ',
                _ => panic!("string.ljust: fill must be a char or nil"),
            };

            let len = inner.0.chars().count();
            if len >= width {
                return Value::String(inner.clone());
            }
            let result = format!("{}{}", inner.0, fill.to_string().repeat(width - len));
            Value::String(TString::new(result))
        } else {
            panic!("Can only use string.ljust on strings");
        }
    }

    /// rjust(width) or rjust(width, fill_char) -> string
    fn rjust(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [string, fill_val, width_val] = get_args!(args, 3);

        if let Value::String(inner) = string {
            let width = if let Value::Number(n) = width_val {
                n as usize
            } else {
                panic!("string.rjust: width must be a number");
            };

            let fill = match fill_val {
                Value::Char(c) => c,
                Value::NIL => ' ',
                _ => panic!("string.rjust: fill must be a char or nil"),
            };

            let len = inner.0.chars().count();
            if len >= width {
                return Value::String(inner.clone());
            }
            let result = format!("{}{}", fill.to_string().repeat(width - len), inner.0);
            Value::String(TString::new(result))
        } else {
            panic!("Can only use string.rjust on strings");
        }
    }

    // --- Predicate / is_* ---

    /// is_upper() -> bool  (non-empty and all cased chars are uppercase)
    fn is_upper(_vm: &mut VM, args: Vec<Value>) -> Value {
        let string = get_args!(args);

        if let Value::String(inner) = string {
            let has_cased = inner.0.chars().any(|c| c.is_alphabetic());
            Value::Bool(has_cased && inner.0.chars().all(|c| !c.is_lowercase()))
        } else {
            panic!("Can only use string.is_upper on strings");
        }
    }

    /// is_lower() -> bool  (non-empty and all cased chars are lowercase)
    fn is_lower(_vm: &mut VM, args: Vec<Value>) -> Value {
        let string = get_args!(args);

        if let Value::String(inner) = string {
            let has_cased = inner.0.chars().any(|c| c.is_alphabetic());
            Value::Bool(has_cased && inner.0.chars().all(|c| !c.is_uppercase()))
        } else {
            panic!("Can only use string.is_lower on strings");
        }
    }

    /// is_numeric() -> bool  (all chars are numeric / digit)
    fn is_numeric(_vm: &mut VM, args: Vec<Value>) -> Value {
        let string = get_args!(args);

        if let Value::String(inner) = string {
            Value::Bool(!inner.0.is_empty() && inner.0.chars().all(|c| c.is_numeric()))
        } else {
            panic!("Can only use string.is_numeric on strings");
        }
    }

    /// is_alphanumeric() -> bool
    fn is_alphanumeric(_vm: &mut VM, args: Vec<Value>) -> Value {
        let string = get_args!(args);

        if let Value::String(inner) = string {
            Value::Bool(!inner.0.is_empty() && inner.0.chars().all(|c| c.is_alphanumeric()))
        } else {
            panic!("Can only use string.is_alphanumeric on strings");
        }
    }

    /// is_alpha() -> bool  (all chars are alphabetic)
    fn is_alpha(_vm: &mut VM, args: Vec<Value>) -> Value {
        let string = get_args!(args);

        if let Value::String(inner) = string {
            Value::Bool(!inner.0.is_empty() && inner.0.chars().all(|c| c.is_alphabetic()))
        } else {
            panic!("Can only use string.is_alpha on strings");
        }
    }

    /// is_ascii() -> bool  (all chars are ASCII)
    fn is_ascii(_vm: &mut VM, args: Vec<Value>) -> Value {
        let string = get_args!(args);

        if let Value::String(inner) = string {
            Value::Bool(inner.0.is_ascii())
        } else {
            panic!("Can only use string.is_ascii on strings");
        }
    }

    /// is_empty() -> bool
    fn is_empty(_vm: &mut VM, args: Vec<Value>) -> Value {
        let string = get_args!(args);

        if let Value::String(inner) = string {
            Value::Bool(inner.0.is_empty())
        } else {
            panic!("Can only use string.is_empty on strings");
        }
    }

    /// is_whitespace() -> bool  (non-empty and all chars are whitespace)
    fn is_whitespace(_vm: &mut VM, args: Vec<Value>) -> Value {
        let string = get_args!(args);

        if let Value::String(inner) = string {
            Value::Bool(!inner.0.is_empty() && inner.0.chars().all(|c| c.is_whitespace()))
        } else {
            panic!("Can only use string.is_whitespace on strings");
        }
    }
}

// LIBRARY
impl Library for StringLib {
    fn get_name(&self) -> &str {
        "string"
    }

    fn get_function(&self, name: u64) -> Box<dyn Fn(&mut VM, Vec<Value>) -> Value> {
        match name {
            // Original
            x if x == hash_u64!("len") => Box::new(Self::len),
            x if x == hash_u64!("concat") => Box::new(Self::concat),
            x if x == hash_u64!("copy") => Box::new(Self::copy),
            x if x == hash_u64!("count") => Box::new(Self::count),
            x if x == hash_u64!("reverse") => Box::new(Self::reverse),
            x if x == hash_u64!("rep") => Box::new(Self::rep),
            x if x == hash_u64!("bytes") => Box::new(Self::bytes),
            x if x == hash_u64!("chars") => Box::new(Self::chars),
            x if x == hash_u64!("split") => Box::new(Self::split),
            x if x == hash_u64!("upper") => Box::new(Self::upper),
            x if x == hash_u64!("lower") => Box::new(Self::lower),
            // Trim
            x if x == hash_u64!("trim") => Box::new(Self::trim),
            x if x == hash_u64!("ltrim") => Box::new(Self::ltrim),
            x if x == hash_u64!("rtrim") => Box::new(Self::rtrim),
            // Search & Replace
            x if x == hash_u64!("replace") => Box::new(Self::replace),
            x if x == hash_u64!("find") => Box::new(Self::find),
            x if x == hash_u64!("starts_with") => Box::new(Self::starts_with),
            x if x == hash_u64!("ends_with") => Box::new(Self::ends_with),
            x if x == hash_u64!("join") => Box::new(Self::join),
            // Case
            x if x == hash_u64!("title") => Box::new(Self::title),
            // Padding
            x if x == hash_u64!("center") => Box::new(Self::center),
            x if x == hash_u64!("ljust") => Box::new(Self::ljust),
            x if x == hash_u64!("rjust") => Box::new(Self::rjust),
            // Predicates
            x if x == hash_u64!("is_upper") => Box::new(Self::is_upper),
            x if x == hash_u64!("is_lower") => Box::new(Self::is_lower),
            x if x == hash_u64!("is_numeric") => Box::new(Self::is_numeric),
            x if x == hash_u64!("is_alphanumeric") => Box::new(Self::is_alphanumeric),
            x if x == hash_u64!("is_alpha") => Box::new(Self::is_alpha),
            x if x == hash_u64!("is_ascii") => Box::new(Self::is_ascii),
            x if x == hash_u64!("is_empty") => Box::new(Self::is_empty),
            x if x == hash_u64!("is_whitespace") => Box::new(Self::is_whitespace),

            _ => panic!("Unknown function `{name}` on lib {}", self.get_name()),
        }
    }
}
