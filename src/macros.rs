#![macro_use]

#[macro_export]
macro_rules! rc {
    ($x: expr) => {
        std::rc::Rc::new($x)
    };
}

#[macro_export]
macro_rules! rc_str {
    ($x:expr) => {
        std::rc::Rc::<str>::from($x)
    };
}

#[macro_export]
macro_rules! hash_u64 {
    ($s: expr) => {{ crate::macros::stable_hash_u64($s) }};
}

pub const fn stable_hash_u64(s: &str) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let bytes = s.as_bytes();
    let mut hash = FNV_OFFSET;
    let mut i = 0;

    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
        i += 1;
    }

    hash
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
    ($this:expr, $lib:expr, $member:expr, $val:expr) => {
        Value::Function(TFunction::with_lib(
            rc_str!($lib),
            rc_str!($member),
            Some(Box::new({ $val }($this.clone()))),
        ))
    };

    ($lib:literal, $member:expr) => {
        Value::Function(TFunction::with_lib(rc_str!($lib), rc_str!($member), None))
    };

    ($lib:expr, $member:expr) => {
        Value::Function(TFunction::with_lib(rc_str!($lib), rc_str!($member), None))
    };
}
