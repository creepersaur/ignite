#[macro_export]
macro_rules! rc {
    ($x: expr) => {
        std::rc::Rc::new($x)
    };
}

#[macro_export]
macro_rules! hashmap {
	{$($key:expr => $value:expr),*} => {{
		let x = std::collections::HashMap::new();

		$(
			x.insert($key, $value);
		)*

		x
	}}
}

#[macro_export]
macro_rules! lib_function {
    ($this:expr, $lib:expr, $member:expr, $args:expr, $val:expr) => {
        Value::Function(TFunction::with_lib(
            rc!($lib.to_string()),
            rc!((*$member.borrow()).clone()),
            $args,
            Some(Box::new({ $val }($this.clone()))),
        ))
    };
}
