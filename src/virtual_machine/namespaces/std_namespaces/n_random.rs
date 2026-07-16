use crate::{
    namespace_lib_function,
    virtual_machine::{
        namespaces::namespace::TNamespace, types::function::TFunction, value::Value,
    },
};
use std::cell::RefCell;

pub fn std_random() -> Value {
    let mut namespace = TNamespace::new("Random", true);

	namespace_lib_function!(namespace, "int");
    namespace_lib_function!(namespace, "uint");
    namespace_lib_function!(namespace, "int_range");

    namespace_lib_function!(namespace, "float");
    namespace_lib_function!(namespace, "float_range");

    Value::Namespace(rc!(RefCell::new(namespace)))
}
