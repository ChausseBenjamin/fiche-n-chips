use std::{
	fmt::{Debug, Display},
	sync::Arc,
};

use crate::logvalue::LogValue;

#[derive(Debug)]
pub enum Attr {
	Obj(AttrLeaf),
	Grp(AttrGroup),
}

#[derive(Debug)]
pub struct AttrGroup {
	key: Arc<str>,
	attrs: Arc<[Attr]>,
}

pub struct AttrLeaf {
	key: Arc<str>,
	value: Box<dyn Display>,
}

impl Attr {
	pub fn group(key: Arc<str>, attrs: Arc<[Self]>) -> Self {
		Attr::Grp(AttrGroup { key, attrs })
	}
	pub fn object(key: Arc<str>, value: Box<dyn Display>) -> Self {
		Attr::Obj(AttrLeaf { key, value })
	}
}

macro_rules! attrs {
    ($($args:tt)*) => {{
        let mut __v: Vec<$crate::attr::Attr> = Vec::new();
        $crate::__attrs_inner!(__v; $($args)*);
        std::sync::Arc::from(__v.as_slice())
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __attrs_inner {
    ($acc:ident;) => {};

    // Key-value pair
    ($acc:ident; $key:literal => $val:expr, $($rest:tt)*) => {
        $acc.push($crate::attr::Attr::object($key, Box::new($crate::log_val!($val))));
        $crate::__attrs_inner!($acc; $($rest)*)
    };

    // Standalone/complete Attr
    ($acc:ident; $attr:expr, $($rest:tt)*) => {
        $Acc.push($attr);
        $crate::__attrs_inner!($acc; $($rest)*)
    };
}

macro_rules! group {
    ($key:literal, $($rest:tt)*) => {
        $crate::attr::Attr::group($key, $crate::attr::attrs!($($rest)*))
    };
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::log_val;

	fn test_flat_list() {
		let output = attrs!(
				"user_id" => 12,
				"name" => "John",
				"surname" => "Doe",
		);
		let expected = Arc::new([
			Attr::Obj(AttrLeaf {
				key: Arc::new("user_id".into()),
				value: Box::new(log_val!(12)),
			}),
			Attr::Obj(AttrLeaf {
				key: Arc::new("name".into()),
				value: Box::new(log_val!("John")),
			}),
			Attr::Obj(AttrLeaf {
				key: Arc::new("surname".into()),
				value: Box::new(log_val!("Doe")),
			}),
		]);
		assert_eq!(output, expected)
	}
}

// TODO: attrs! macro that can receive a mixed list 3 things:
// - &str
// - Box<dyn Display>
// - Attr
// Its goal is pre-compile an Arc<[Attr]>, to abstract away the enum wrapping
// in the process. It expects every &str (assumed to be a key) to be followed
// by a non-Attr object (value). It will call log_val!() on the value and then
// merge key+value into an Attr to add to the list.

// TODO: group! macro that does the following:
// - use the first argument as a key (must be &str) for an Attr::Grp()
// - pass all remaining arguments to the attrs! macro to get the Arc<[Attr]> for
//   that group.
