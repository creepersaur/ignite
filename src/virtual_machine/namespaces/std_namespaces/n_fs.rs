use crate::{
    namespace_lib_function,
    virtual_machine::{
        namespaces::namespace::TNamespace, types::function::TFunction, value::Value,
    },
};
use std::cell::RefCell;

pub fn std_fs() -> Value {
    let mut namespace = TNamespace::new("FS", true);

    namespace_lib_function!(namespace, "read");
    namespace_lib_function!(namespace, "write");

    Value::Namespace(rc!(RefCell::new(namespace)))
}
