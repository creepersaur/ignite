use crate::virtual_machine::{value::Value, vm::VM};

pub trait Library {
    fn get_name(&self) -> &str;

    fn get_function(&self, name: u64) -> Box<dyn Fn(&mut VM, Vec<Value>) -> Value>;
}

#[macro_export]
macro_rules! get_args {
	($args:expr, $count: expr) => {{
		let v: [Value; $count] = $args.try_into().expect(&format!("Expected {} arguments", $count));
		v
	}};

	($args: expr) => {
		&$args[0]
	}
}