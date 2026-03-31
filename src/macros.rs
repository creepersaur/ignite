#[macro_export]
macro_rules! rc {
    ($x: literal) => {
        std::rc::Rc::from($x)
    };
    ($x: expr) => {
        std::rc::Rc::new($x)
    };
}

#[macro_export]
macro_rules! hash_u64 {
    ($s: expr) => {{
        use rustc_hash::FxHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = FxHasher::default();
        $s.hash(&mut hasher);
        hasher.finish()
    }};
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
            rc!($lib),
            $member,
            $args,
            Some(Box::new({ $val }($this.clone()))),
        ))
    };

    ($lib:expr, $member:expr, $args:expr, $val:expr) => {
        Value::Function(TFunction::with_lib(
            rc!($lib),
            rc!($member),
            $args,
            None,
        ))
    };
}
