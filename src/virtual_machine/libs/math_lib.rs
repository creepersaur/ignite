use crate::{
    get_args, hash_u64, virtual_machine::{libs::lib::Library, value::Value, vm::VM}
};

pub struct MathLib;

impl MathLib {
    fn num(v: &Value, fn_name: &str) -> f64 {
        match v {
            Value::Number(x) => *x,
            _ => panic!("math.{fn_name} expects a number"),
        }
    }

    // Basic
    fn abs(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "abs").abs())
    }
    fn ceil(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "ceil").ceil())
    }
    fn floor(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "floor").floor())
    }
    fn round(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "round").round())
    }
    fn trunc(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "trunc").trunc())
    }
    fn fract(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "fract").fract())
    }
    fn sign(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "sign").signum())
    }
    fn sqrt(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "sqrt").sqrt())
    }
    fn cbrt(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "cbrt").cbrt())
    }
    fn exp(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "exp").exp())
    }
    fn exp2(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "exp2").exp2())
    }
    fn ln(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "ln").ln())
    }
    fn log2(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "log2").log2())
    }
    fn log10(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "log10").log10())
    }
    fn recip(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "recip").recip())
    }

    // Two-argument
    fn pow(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [base, exp] = get_args!(args, 2);
        Value::Number(Self::num(&base, "pow").powf(Self::num(&exp, "pow")))
    }
    fn log(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [x, base] = get_args!(args, 2);
        Value::Number(Self::num(&x, "log").log(Self::num(&base, "log")))
    }
    fn hypot(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [a, b] = get_args!(args, 2);
        Value::Number(Self::num(&a, "hypot").hypot(Self::num(&b, "hypot")))
    }
    fn atan2(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [y, x] = get_args!(args, 2);
        Value::Number(Self::num(&y, "atan2").atan2(Self::num(&x, "atan2")))
    }
    fn min(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [a, b] = get_args!(args, 2);
        Value::Number(Self::num(&a, "min").min(Self::num(&b, "min")))
    }
    fn max(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [a, b] = get_args!(args, 2);
        Value::Number(Self::num(&a, "max").max(Self::num(&b, "max")))
    }
    fn clamp(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [x, min, max] = get_args!(args, 3);
        Value::Number(Self::num(&x, "clamp").clamp(Self::num(&min, "clamp"), Self::num(&max, "clamp")))
    }
    fn copysign(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [x, sign] = get_args!(args, 2);
        Value::Number(Self::num(&x, "copysign").copysign(Self::num(&sign, "copysign")))
    }

    // Trig
    fn sin(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "sin").sin())
    }
    fn cos(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "cos").cos())
    }
    fn tan(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "tan").tan())
    }
    fn sinh(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "sinh").sinh())
    }
    fn cosh(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "cosh").cosh())
    }
    fn tanh(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "tanh").tanh())
    }

    // Inverse trig
    fn asin(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "asin").asin())
    }
    fn acos(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "acos").acos())
    }
    fn atan(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "atan").atan())
    }
    fn asinh(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "asinh").asinh())
    }
    fn acosh(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "acosh").acosh())
    }
    fn atanh(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "atanh").atanh())
    }

    // Conversion
    fn to_radians(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "to_radians").to_radians())
    }
    fn to_degrees(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number(Self::num(&x, "to_degrees").to_degrees())
    }
    fn to_celsius(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number((Self::num(&x, "to_celsius") - 32.0) * 5.0 / 9.0)
    }
    fn to_fahrenheit(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Number((Self::num(&x, "to_fahrenheit") * 9.0 / 5.0) + 32.0)
    }

    // Predicates
    fn is_nan(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Bool(Self::num(&x, "is_nan").is_nan())
    }
    fn is_infinite(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Bool(Self::num(&x, "is_infinite").is_infinite())
    }
    fn is_finite(_vm: &mut VM, args: Vec<Value>) -> Value {
        let x = get_args!(args);
        Value::Bool(Self::num(&x, "is_finite").is_finite())
    }

    // Rounding
    fn round_to(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [x, decimals] = get_args!(args, 2);
        let factor = 10f64.powi(Self::num(&decimals, "round_to") as i32);
        Value::Number((Self::num(&x, "round_to") * factor).round() / factor)
    }

    // Interpolation
    fn lerp(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [a, b, t] = get_args!(args, 3);
        let (a, b, t) = (
            Self::num(&a, "lerp"),
            Self::num(&b, "lerp"),
            Self::num(&t, "lerp"),
        );
        Value::Number(a + (b - a) * t)
    }
    fn inv_lerp(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [a, b, x] = get_args!(args, 3);
        let (a, b, x) = (
            Self::num(&a, "inv_lerp"),
            Self::num(&b, "inv_lerp"),
            Self::num(&x, "inv_lerp"),
        );
        Value::Number((x - a) / (b - a))
    }
    fn remap(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [x, in_min, in_max, out_min, out_max] = get_args!(args, 5);
        let (x, in_min, in_max, out_min, out_max) = (
            Self::num(&x, "remap"),
            Self::num(&in_min, "remap"),
            Self::num(&in_max, "remap"),
            Self::num(&out_min, "remap"),
            Self::num(&out_max, "remap"),
        );
        let t = (x - in_min) / (in_max - in_min);
        Value::Number(out_min + t * (out_max - out_min))
    }
    fn smoothstep(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [edge0, edge1, x] = get_args!(args, 3);
        let (edge0, edge1, x) = (
            Self::num(&edge0, "smoothstep"),
            Self::num(&edge1, "smoothstep"),
            Self::num(&x, "smoothstep"),
        );
        let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
        Value::Number(t * t * (3.0 - 2.0 * t))
    }

    // Number theory
    fn gcd(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [a, b] = get_args!(args, 2);
        let (mut a, mut b) = (Self::num(&a, "gcd") as u64, Self::num(&b, "gcd") as u64);
        while b != 0 {
            let t = b;
            b = a % b;
            a = t;
        }
        Value::Number(a as f64)
    }
    fn lcm(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [a, b] = get_args!(args, 2);
        let (a, b) = (Self::num(&a, "lcm") as u64, Self::num(&b, "lcm") as u64);
        let (mut ta, mut tb) = (a, b);
        while tb != 0 {
            let t = tb;
            tb = ta % tb;
            ta = t;
        }
        Value::Number((a / ta * b) as f64)
    }
    fn factorial(_vm: &mut VM, args: Vec<Value>) -> Value {
        let n = get_args!(args);
        let n = Self::num(&n, "factorial") as u64;
        Value::Number((1..=n).product::<u64>() as f64)
    }
    fn is_prime(_vm: &mut VM, args: Vec<Value>) -> Value {
        let n = get_args!(args);
        let n = Self::num(&n, "is_prime") as u64;
        if n < 2 {
            return Value::Bool(false);
        }
        if n == 2 {
            return Value::Bool(true);
        }
        if n % 2 == 0 {
            return Value::Bool(false);
        }
        let limit = (n as f64).sqrt() as u64;
        Value::Bool((3..=limit).step_by(2).all(|i| n % i != 0))
    }

    // Numeric utilities
    fn fma(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [a, b, c] = get_args!(args, 3);
        Value::Number(Self::num(&a, "fma").mul_add(Self::num(&b, "fma"), Self::num(&c, "fma")))
    }
    fn mid(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [a, b] = get_args!(args, 2);
        Value::Number((Self::num(&a, "mid") + Self::num(&b, "mid")) / 2.0)
    }
    fn wrap(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [x, min, max] = get_args!(args, 3);
        let (x, min, max) = (
            Self::num(&x, "wrap"),
            Self::num(&min, "wrap"),
            Self::num(&max, "wrap"),
        );
        let range = max - min;
        Value::Number(min + ((x - min) % range + range) % range)
    }
    fn snap(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [x, step] = get_args!(args, 2);
        Value::Number(
            (Self::num(&x, "snap") / Self::num(&&step, "snap")).round() * Self::num(&step, "snap"),
        )
    }
    fn ping_pong(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [x, length] = get_args!(args, 2);
        let (x, length) = (Self::num(&x, "ping_pong"), Self::num(&length, "ping_pong"));
        let t = x % (length * 2.0);
        Value::Number(if t > length { length * 2.0 - t } else { t })
    }

    // Geometry
    fn dist(_vm: &mut VM, args: Vec<Value>) -> Value {
        let [x1, y1, x2, y2] = get_args!(args, 4);
        let (x1, y1, x2, y2) = (
            Self::num(&x1, "dist"),
            Self::num(&y1, "dist"),
            Self::num(&x2, "dist"),
            Self::num(&y2, "dist"),
        );
        Value::Number((x2 - x1).hypot(y2 - y1))
    }
}

// LIBRARY
impl Library for MathLib {
    fn get_name(&self) -> &str {
        "math"
    }

    fn get_function(&self, name: u64) -> Box<dyn Fn(&mut VM, Vec<Value>) -> Value> {
        match name {
            // Basic
            x if x == hash_u64!("abs") => Box::new(Self::abs),
            x if x == hash_u64!("ceil") => Box::new(Self::ceil),
            x if x == hash_u64!("floor") => Box::new(Self::floor),
            x if x == hash_u64!("round") => Box::new(Self::round),
            x if x == hash_u64!("trunc") => Box::new(Self::trunc),
            x if x == hash_u64!("fract") => Box::new(Self::fract),
            x if x == hash_u64!("sign") => Box::new(Self::sign),
            x if x == hash_u64!("sqrt") => Box::new(Self::sqrt),
            x if x == hash_u64!("cbrt") => Box::new(Self::cbrt),
            x if x == hash_u64!("exp") => Box::new(Self::exp),
            x if x == hash_u64!("exp2") => Box::new(Self::exp2),
            x if x == hash_u64!("ln") => Box::new(Self::ln),
            x if x == hash_u64!("log2") => Box::new(Self::log2),
            x if x == hash_u64!("log10") => Box::new(Self::log10),
            x if x == hash_u64!("recip") => Box::new(Self::recip),

            // Two-argument
            x if x == hash_u64!("pow") => Box::new(Self::pow),
            x if x == hash_u64!("log") => Box::new(Self::log),
            x if x == hash_u64!("hypot") => Box::new(Self::hypot),
            x if x == hash_u64!("atan2") => Box::new(Self::atan2),
            x if x == hash_u64!("min") => Box::new(Self::min),
            x if x == hash_u64!("max") => Box::new(Self::max),
            x if x == hash_u64!("clamp") => Box::new(Self::clamp),
            x if x == hash_u64!("copysign") => Box::new(Self::copysign),

            // Trig
            x if x == hash_u64!("sin") => Box::new(Self::sin),
            x if x == hash_u64!("cos") => Box::new(Self::cos),
            x if x == hash_u64!("tan") => Box::new(Self::tan),
            x if x == hash_u64!("sinh") => Box::new(Self::sinh),
            x if x == hash_u64!("cosh") => Box::new(Self::cosh),
            x if x == hash_u64!("tanh") => Box::new(Self::tanh),

            // Inverse trig
            x if x == hash_u64!("asin") => Box::new(Self::asin),
            x if x == hash_u64!("acos") => Box::new(Self::acos),
            x if x == hash_u64!("atan") => Box::new(Self::atan),
            x if x == hash_u64!("asinh") => Box::new(Self::asinh),
            x if x == hash_u64!("acosh") => Box::new(Self::acosh),
            x if x == hash_u64!("atanh") => Box::new(Self::atanh),

            // Conversion
            x if x == hash_u64!("to_radians") => Box::new(Self::to_radians),
            x if x == hash_u64!("to_degrees") => Box::new(Self::to_degrees),
            x if x == hash_u64!("to_celsius") => Box::new(Self::to_celsius),
            x if x == hash_u64!("to_fahrenheit") => Box::new(Self::to_fahrenheit),

            // Predicates
            x if x == hash_u64!("is_nan") => Box::new(Self::is_nan),
            x if x == hash_u64!("is_infinite") => Box::new(Self::is_infinite),
            x if x == hash_u64!("is_finite") => Box::new(Self::is_finite),

            x if x == hash_u64!("round_to") => Box::new(Self::round_to),
            x if x == hash_u64!("lerp") => Box::new(Self::lerp),
            x if x == hash_u64!("inv_lerp") => Box::new(Self::inv_lerp),
            x if x == hash_u64!("remap") => Box::new(Self::remap),
            x if x == hash_u64!("smoothstep") => Box::new(Self::smoothstep),
            x if x == hash_u64!("gcd") => Box::new(Self::gcd),
            x if x == hash_u64!("lcm") => Box::new(Self::lcm),
            x if x == hash_u64!("factorial") => Box::new(Self::factorial),
            x if x == hash_u64!("is_prime") => Box::new(Self::is_prime),
            x if x == hash_u64!("fma") => Box::new(Self::fma),
            x if x == hash_u64!("mid") => Box::new(Self::mid),
            x if x == hash_u64!("wrap") => Box::new(Self::wrap),
            x if x == hash_u64!("snap") => Box::new(Self::snap),
            x if x == hash_u64!("ping_pong") => Box::new(Self::ping_pong),
            x if x == hash_u64!("dist") => Box::new(Self::dist),

            _ => panic!("Unknown function `{name}` on lib {}", self.get_name()),
        }
    }
}
