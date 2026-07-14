use crate::{
    namespace_lib_function,
    virtual_machine::{
        namespaces::namespace::TNamespace, types::function::TFunction, value::Value,
    },
};
use std::cell::RefCell;

pub fn std_io() -> Value {
    let mut namespace = TNamespace::new("IO", true);

    // Input
    namespace_lib_function!(namespace, "read_line");
    namespace_lib_function!(namespace, "read_line_raw");
    namespace_lib_function!(namespace, "read");
    namespace_lib_function!(namespace, "read_key");

    // Output
    namespace_lib_function!(namespace, "clear");
    namespace_lib_function!(namespace, "reset");
    namespace_lib_function!(namespace, "flush");
    namespace_lib_function!(namespace, "write");
    namespace_lib_function!(namespace, "write_line");

    Value::Namespace(rc!(RefCell::new(namespace)))
}
