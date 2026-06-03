use crate::{
    namespace_lib_function,
    virtual_machine::{
        namespaces::namespace::TNamespace, types::function::TFunction, value::Value,
    },
};
use std::cell::RefCell;

pub fn std_math() -> Value {
    let mut namespace = TNamespace::new("Math", true);

    // Constants
    namespace.set_const("PI", Value::Number(std::f64::consts::PI));
    namespace.set_const("E", Value::Number(std::f64::consts::E));
    namespace.set_const("TAU", Value::Number(std::f64::consts::TAU));
    namespace.set_const("SQRT2", Value::Number(std::f64::consts::SQRT_2));
    namespace.set_const("LN2", Value::Number(std::f64::consts::LN_2));
    namespace.set_const("LN10", Value::Number(std::f64::consts::LN_10));
    namespace.set_const("LOG2E", Value::Number(std::f64::consts::LOG2_E));
    namespace.set_const("LOG10E", Value::Number(std::f64::consts::LOG10_E));
    namespace.set_const("INFINITY", Value::Number(f64::INFINITY));
    namespace.set_const("NEG_INFINITY", Value::Number(f64::NEG_INFINITY));
    namespace.set_const("NAN", Value::Number(f64::NAN));

    // Basic
    namespace_lib_function!(namespace, "abs");
    namespace_lib_function!(namespace, "ceil");
    namespace_lib_function!(namespace, "floor");
    namespace_lib_function!(namespace, "trunc");
    namespace_lib_function!(namespace, "fract");
    namespace_lib_function!(namespace, "sign");
    namespace_lib_function!(namespace, "sqrt");
    namespace_lib_function!(namespace, "cbrt");
    namespace_lib_function!(namespace, "exp");
    namespace_lib_function!(namespace, "exp2");
    namespace_lib_function!(namespace, "ln");
    namespace_lib_function!(namespace, "log2");
    namespace_lib_function!(namespace, "log10");
    namespace_lib_function!(namespace, "recip");

    // Two-argument
    namespace_lib_function!(namespace, "pow");
    namespace_lib_function!(namespace, "log"); // log(x, base)
    namespace_lib_function!(namespace, "hypot");
    namespace_lib_function!(namespace, "atan2");
    namespace_lib_function!(namespace, "min");
    namespace_lib_function!(namespace, "max");
    namespace_lib_function!(namespace, "clamp"); // clamp(x, min, max)
    namespace_lib_function!(namespace, "copysign");

    // Trig
    namespace_lib_function!(namespace, "sin");
    namespace_lib_function!(namespace, "cos");
    namespace_lib_function!(namespace, "tan");
    namespace_lib_function!(namespace, "sinh");
    namespace_lib_function!(namespace, "cosh");
    namespace_lib_function!(namespace, "tanh");

    // Inverse trig
    namespace_lib_function!(namespace, "asin");
    namespace_lib_function!(namespace, "acos");
    namespace_lib_function!(namespace, "atan");
    namespace_lib_function!(namespace, "asinh");
    namespace_lib_function!(namespace, "acosh");
    namespace_lib_function!(namespace, "atanh");

    // Conversion
    namespace_lib_function!(namespace, "to_radians");
    namespace_lib_function!(namespace, "to_degrees");
    namespace_lib_function!(namespace, "to_celsius");
    namespace_lib_function!(namespace, "to_fahrenheit");

    // Predicates — return Bool
    namespace_lib_function!(namespace, "is_nan");
    namespace_lib_function!(namespace, "is_infinite");
    namespace_lib_function!(namespace, "is_finite");

    // Rounding
    namespace_lib_function!(namespace, "round");
    namespace_lib_function!(namespace, "round_to");

    namespace_lib_function!(namespace, "lerp");
    namespace_lib_function!(namespace, "inv_lerp");
    namespace_lib_function!(namespace, "remap");
    namespace_lib_function!(namespace, "smoothstep");

    namespace_lib_function!(namespace, "gcd");
    namespace_lib_function!(namespace, "lcm");
    namespace_lib_function!(namespace, "factorial");
    namespace_lib_function!(namespace, "is_prime");

    namespace_lib_function!(namespace, "fma");
    namespace_lib_function!(namespace, "mid");
    namespace_lib_function!(namespace, "wrap");
    namespace_lib_function!(namespace, "snap");
    namespace_lib_function!(namespace, "ping_pong");

    // Geometry
    namespace_lib_function!(namespace, "dist");

    Value::Namespace(rc!(RefCell::new(namespace)))
}
