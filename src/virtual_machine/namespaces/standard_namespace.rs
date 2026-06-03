use crate::virtual_machine::{
    namespaces::{
        namespace::TNamespace,
        std_namespaces::{n_fs::std_fs, n_io::std_io, n_math::std_math},
    },
    value::Value,
};
use std::cell::RefCell;

pub fn load_standard_namespace() -> Value {
    let mut namespace = TNamespace::new("Std", true);

    namespace.env.insert(rc_str!("Math"), (std_math(), true));
    namespace.env.insert(rc_str!("IO"), (std_io(), true));
    namespace.env.insert(rc_str!("FS"), (std_fs(), true));

    return Value::Namespace(rc!(RefCell::new(namespace)));
}
