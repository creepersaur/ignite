use bincode::{Decode, Encode};
use std::{
    cell::RefCell,
    fmt::Debug,
    hash::{Hash, Hasher},
    rc::Rc,
};

use crate::virtual_machine::{
    libs::types::TypeValue,
    namespaces::namespace::TNamespace,
    types::{
        classes::{r#class::TClass, class_object::TClassObject},
        dict::TDict,
        r#enum::TEnum,
        function::TFunction,
        list::TList,
        string::TString,
        r#struct::TStruct,
        structdef::TStructDef,
    },
};

#[allow(unused)]
#[derive(Encode, Decode, Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    NIL,
    Type(TypeValue),

    Number(f64),
    Bool(bool),
    Char(char),
    String(TString),

    Function(Box<TFunction>),

    // Collections
    List(TList),
    Tuple(TList),
    Dict(TDict),

    StructDef(Rc<TStructDef>),
    Struct(TStruct),

    Class(Rc<RefCell<TClass>>),
    ClassObject(TClassObject),

    // Namespaces
    Namespace(Rc<RefCell<TNamespace>>),
    Enum(TEnum),

    Range {
        start: Box<Value>,
        end: Box<Value>,
        step: Box<Value>,
        inclusive: bool,
    },
}

impl ToString for Value {
    fn to_string(&self) -> String {
        self.to_string(false)
    }
}

impl ToString for &Value {
    fn to_string(&self) -> String {
        (*self).to_string(false)
    }
}

impl Value {
    pub fn string(x: impl ToString) -> Value {
        Value::String(TString::new(x.to_string()))
    }

    pub fn get_type(&self) -> String {
        let t = match self {
            Value::NIL => "nil",
            Value::Type(_) => "type",
            Value::Number(_) => "number",
            Value::Bool(_) => "bool",
            Value::Char(_) => "char",
            Value::String(_) => "string",
            Value::Function(_) => "function",
            Value::List(_) => "list",
            Value::Tuple(_) => "tuple",
            Value::Dict(_) => "dict",
            Value::Namespace(_) => "namespace",
            Value::Class(data) => &*data.borrow().name,
            Value::ClassObject(data) => &data.base.borrow().name,
            Value::Enum(_) => "enum",
            Value::Range { .. } => "range",
            Value::StructDef(..) => "structdef",
            Value::Struct(data) => &data.base.name,
        };
        t.to_owned()
    }

    pub fn to_string(&self, debug: bool) -> String {
        match self {
            Self::NIL => "nil".to_string(),
            Self::Type(x) => format!("{x:?}"),
            Self::Number(x) => x.to_string(),
            Self::Bool(x) => x.to_string(),
            Self::Char(x) => {
                if debug {
                    format!("'{}'", x.to_string())
                } else {
                    x.to_string()
                }
            }
            Self::String(x) => {
                if debug {
                    format!("\"{}\"", x.0.replace("\r", ""))
                } else {
                    x.0.to_string().replace("\r", "")
                }
            }

            Self::Function(f) => {
                let id = match f.handler {
                    Some((lib, method)) => lib ^ method,
                    None => {
                        let mut hasher = std::collections::hash_map::DefaultHasher::new();
                        use std::hash::Hash;
                        f.entry.hash(&mut hasher);
                        std::hash::Hasher::finish(&hasher)
                    }
                };
                format!("<function: 0x{:x}>", id)
            }

            Self::List(list) => format!(
                "[{}]",
                list.values
                    .borrow()
                    .iter()
                    .map(|x| if let Value::List(v) = x {
                        if list.values.as_ptr() == v.values.as_ptr() {
                            String::from("[...]")
                        } else {
                            x.to_string(true)
                        }
                    } else {
                        x.to_string(true)
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Tuple(list) => format!(
                "({})",
                list.values
                    .borrow()
                    .iter()
                    .map(|x| if let Value::List(v) = x {
                        if list.values.as_ptr() == v.values.as_ptr() {
                            String::from("(...)")
                        } else {
                            x.to_string(true)
                        }
                    } else {
                        x.to_string(true)
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Dict(dict) => format!(
                "{{{}}}",
                dict.values
                    .borrow()
                    .iter()
                    .map(|(k, v)| {
                        let key = if let Value::Dict(key) = k {
                            if dict.values.as_ptr() == key.values.as_ptr() {
                                String::from("{...}")
                            } else {
                                k.to_string(true)
                            }
                        } else {
                            k.to_string(true)
                        };

                        let value = if let Value::Dict(val) = v {
                            if dict.values.as_ptr() == val.values.as_ptr() {
                                String::from("{...}")
                            } else {
                                v.to_string(true)
                            }
                        } else {
                            v.to_string(true)
                        };

                        format!("{key}: {value}")
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),

            Self::Namespace(space) => format!("namespace:{}", space.borrow().name),

            Self::Enum(e) => format!("enum:{}", e.name),

            Self::Range {
                start,
                end,
                step,
                inclusive,
            } => format!(
                "Range<{}..{}{}..{}>",
                start.to_string(true),
                if *inclusive { "=" } else { "" },
                end.to_string(true),
                step.to_string(true),
            ),

            Value::StructDef(def) => format!("structdef:{}", def.name),
            Value::Struct(data) => format!(
                "{} {{ {} }}",
                data.base.name,
                data.values
                    .borrow()
                    .iter()
                    .map(|(k, v)| {
                        let value = if let Value::Struct(val) = v {
                            if data.values.as_ptr() == val.values.as_ptr() {
                                String::from("{...}")
                            } else {
                                v.to_string(true)
                            }
                        } else {
                            v.to_string(true)
                        };

                        format!("{k}: {value}")
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),

            Value::Class(def) => format!("class:{}", def.borrow().name),
            Value::ClassObject(data) => format!("object:{}", data.base.borrow().name),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Self::NIL => false,
            Self::Bool(x) => *x,
            _ => true,
        }
    }

    pub fn as_number(&self) -> f64 {
        if let Value::Number(x) = self {
            *x
        } else {
            panic!("Cannot convert `{self:?}` to number.")
        }
    }

    pub fn as_str(&self) -> &str {
		if let Value::String(x) = self {
			&x.0
		} else {
			panic!("Cannot covnert `{self:?}` to str")
		}
	}

    pub fn type_matches(&self, type_hint: &str) -> bool {
        match type_hint {
            "number" => matches!(self, Value::Number(_)),
            "string" => matches!(self, Value::String(_)),
            "bool" => matches!(self, Value::Bool(_)),
            "char" => matches!(self, Value::Char(_)),
            "list" => matches!(self, Value::List(_)),
            "dict" => matches!(self, Value::Dict(_)),
            "any" => true,

            _ => match self {
                Value::Class(def) => &*def.borrow().name == type_hint,
                Value::ClassObject(data) => &*data.base.borrow().name == type_hint,

                Value::StructDef(def) => &*def.name == type_hint,
                Value::Struct(data) => &*data.base.name == type_hint,

                _ => false,
            },
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);

        match self {
            Self::NIL => (),
            Self::Type(x) => x.hash(state),
            Self::Number(n) => {
                // We convert to bits to provide a consistent hash.
                n.to_bits().hash(state);
            }
            Self::Bool(b) => b.hash(state),
            Self::Char(b) => b.hash(state),
            Self::String(s) => s.0.hash(state),
            Self::Function(f) => f.hash(state),

            // For Rc/RefCell types, you usually hash the pointer address
            Self::List(l) => {
                std::ptr::hash(l.values.as_ptr(), state);
            }
            Self::Tuple(l) => {
                std::ptr::hash(l.values.as_ptr(), state);
            }
            Self::Dict(d) => {
                std::ptr::hash(d.values.as_ptr(), state);
            }

            Self::Namespace(space) => space.borrow().hash(state),

            Self::Enum(e) => std::ptr::hash(e.values.as_ref(), state),

            Self::Range {
                start,
                end,
                step,
                inclusive,
            } => {
                start.hash(state);
                end.hash(state);
                step.hash(state);
                inclusive.hash(state);
            }

            Self::StructDef(def) => {
                def.name.hash(state);
                std::ptr::hash(Rc::as_ptr(&def.fields), state);
            }
            Self::Struct(data) => {
                data.base.name.hash(state);
                data.values.as_ptr().hash(state);
            }

            Self::Class(def) => {
                def.borrow().name.hash(state);
                std::ptr::hash(Rc::as_ptr(&def.borrow().values), state);
                std::ptr::hash(Rc::as_ptr(&def.borrow().functions), state);
            }
            Self::ClassObject(data) => {
                data.base.borrow().name.hash(state);
                data.values.as_ptr().hash(state);
            }
        }
    }
}
