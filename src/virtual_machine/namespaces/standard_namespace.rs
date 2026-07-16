use crate::virtual_machine::{
    namespaces::{
        namespace::TNamespace, std_namespaces::{n_fs::std_fs, n_io::std_io, n_math::std_math, n_random::std_random},
    }, value::Value,
};
use std::cell::RefCell;

pub fn load_standard_namespace() -> Value {
    let mut namespace = TNamespace::new("Std", true);

    namespace.env.insert(hash_u64!("Math"), (std_math(), true));
    namespace.env.insert(hash_u64!("IO"), (std_io(), true));
    namespace.env.insert(hash_u64!("FS"), (std_fs(), true));
    namespace.env.insert(hash_u64!("Random"), (std_random(), true));

    return Value::Namespace(rc!(RefCell::new(namespace)));
}
