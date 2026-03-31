use crate::{
    rc,
    virtual_machine::{
        namespaces::{namespace::TNamespace, std_namespaces::n_math::std_math},
        value::Value,
    },
};
use std::cell::RefCell;

pub fn load_standard_namespace() -> Value {
    let mut namespace = TNamespace::new("Std", true);

    namespace
        .env
        .insert(rc!("Math"), (std_math(), true));

    return Value::Namespace(rc!(RefCell::new(namespace)));
}
