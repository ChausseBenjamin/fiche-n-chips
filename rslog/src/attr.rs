use std::{
	fmt::{Debug, Display},
	sync::Arc,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Attr {
	Obj(AttrLeaf),
	Grp(AttrGroup),
}

#[derive(Debug, PartialEq, Clone)]
pub struct AttrGroup {
	pub key: Arc<str>,
	pub attrs: Arc<[Attr]>,
}

#[derive(Clone)]
pub struct AttrLeaf {
	pub key: Arc<str>,
	pub value: Box<dyn Display>,
}

impl std::fmt::Debug for AttrLeaf {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"AttrLeaf {{ key: {:?}, value: {} }}",
			self.key, self.value
		)
	}
}

impl PartialEq for AttrLeaf {
	fn eq(&self, other: &Self) -> bool {
		let mut fail = false;
		fail |= self.key != other.key;
		fail |= format!("{}", self.value) != format!("{}", other.value);
		!fail
	}
}

impl Attr {
	pub fn group(key: Arc<str>, attrs: Arc<[Self]>) -> Self {
		Attr::Grp(AttrGroup { key, attrs })
	}
	pub fn object(key: Arc<str>, value: Box<dyn Display>) -> Self {
		Attr::Obj(AttrLeaf { key, value })
	}
}

#[macro_export]
macro_rules! attrs {
    ($($args:tt)*) => {{
        let mut __v: Vec<$crate::attr::Attr> = Vec::new();
        $crate::__attrs_inner!(__v; $($args)*);
        std::sync::Arc::from(__v.into_boxed_slice())
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __attrs_inner {
    // Allow trailing comma
    ($acc:ident; ,) => {};
    ($acc:ident; , $($rest:tt)*) => { $crate::__attrs_inner!($acc; $($rest)*) };
    ($acc:ident;) => {};

    // Key-value pair at end
    ($acc:ident; $key:literal => $val:expr) => {
        $acc.push($crate::attr!($key, $val));
    };

    // Attr expr at end
    ($acc:ident; $attr:expr) => {
        $acc.push($attr);
    };

    // Key-value pair
    ($acc:ident; $key:literal => $val:expr, $($rest:tt)*) => {
        $acc.push($crate::attr!($key, $val));
        $crate::__attrs_inner!($acc; $($rest)*)
    };

    // Standalone/complete Attr
    ($acc:ident; $attr:expr, $($rest:tt)*) => {
        $acc.push($attr);
        $crate::__attrs_inner!($acc; $($rest)*)
    };
}

#[macro_export]
macro_rules! group {
    ($key:literal, $($rest:tt)*) => {
        $crate::attr::Attr::group($key.into(), $crate::attrs!($($rest)*))
    };
}

#[macro_export]
macro_rules! attr {
	($key:literal, $val:expr) => {
		$crate::attr::Attr::object($key.into(), Box::new($crate::log_val!($val)))
	};
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::log_val;

	#[test]
	fn attr_macro_single() {
		let attr = attr!("foo", 42);
		let expected = Attr::Obj(AttrLeaf {
			key: Arc::from("foo"),
			value: Box::new(log_val!(42)),
		});
		assert_eq!(attr, expected);
	}

	#[test]
	fn homogeneous_macro() {
		let output = attrs!(
				"user_id" => 12,
				"name" => "John",
				"surname" => "Doe",
		);
		let expected = Arc::<[Attr]>::from(
			vec![
				Attr::Obj(AttrLeaf {
					key: Arc::from("user_id"),
					value: Box::new(log_val!(12)),
				}),
				Attr::Obj(AttrLeaf {
					key: Arc::from("name"),
					value: Box::new(log_val!("John")),
				}),
				Attr::Obj(AttrLeaf {
					key: Arc::from("surname"),
					value: Box::new(log_val!("Doe")),
				}),
			]
			.into_boxed_slice(),
		);
		assert_eq!(output, expected)
	}

	#[test]
	fn heterogeneous_macro() {
		let output = attrs!(
				"user_id" => 12,
				Attr::Obj(AttrLeaf{key: Arc::from("name"), value: Box::new(log_val!("John"))}),
				"surname" => "Doe",
		);
		let expected = Arc::<[Attr]>::from(
			vec![
				Attr::Obj(AttrLeaf {
					key: Arc::from("user_id"),
					value: Box::new(log_val!(12)),
				}),
				Attr::Obj(AttrLeaf {
					key: Arc::from("name"),
					value: Box::new(log_val!("John")),
				}),
				Attr::Obj(AttrLeaf {
					key: Arc::from("surname"),
					value: Box::new(log_val!("Doe")),
				}),
			]
			.into_boxed_slice(),
		);
		assert_eq!(output, expected);
	}

	#[test]
	fn attr_macro_group() {
		// Homogeneous
		let output = group!("parent",
			"child1" => 10,
			"child2" => 20
		);
		let expected = Attr::Grp(AttrGroup {
			key: Arc::from("parent"),
			attrs: Arc::<[Attr]>::from(
				vec![
					Attr::Obj(AttrLeaf {
						key: Arc::from("child1"),
						value: Box::new(log_val!(10)),
					}),
					Attr::Obj(AttrLeaf {
						key: Arc::from("child2"),
						value: Box::new(log_val!(20)),
					}),
				]
				.into_boxed_slice(),
			),
		});
		assert_eq!(output, expected);

		// Heterogeneous (mix of kv and attr!)
		let output = group!("parent",
			"child1" => 30,
			attr!("child2", 40)
		);
		let expected = Attr::Grp(AttrGroup {
			key: Arc::from("parent"),
			attrs: Arc::<[Attr]>::from(
				vec![
					Attr::Obj(AttrLeaf {
						key: Arc::from("child1"),
						value: Box::new(log_val!(30)),
					}),
					Attr::Obj(AttrLeaf {
						key: Arc::from("child2"),
						value: Box::new(log_val!(40)),
					}),
				]
				.into_boxed_slice(),
			),
		});
		assert_eq!(output, expected);
	}

	#[test]
	fn combined_macro_usage() {
		let output = attrs!(
			"top_level" => "root",
			attr!("middle", 99),
			group!("outer_group",
				"a" => 1,
				attr!("b", 2),
				group!("inner_group",
					"z" => 3
				)
			)
		);
		let expected = Arc::<[Attr]>::from(
			vec![
				Attr::Obj(AttrLeaf {
					key: Arc::from("top_level"),
					value: Box::new(log_val!("root")),
				}),
				Attr::Obj(AttrLeaf {
					key: Arc::from("middle"),
					value: Box::new(log_val!(99)),
				}),
				Attr::Grp(AttrGroup {
					key: Arc::from("outer_group"),
					attrs: Arc::<[Attr]>::from(
						vec![
							Attr::Obj(AttrLeaf {
								key: Arc::from("a"),
								value: Box::new(log_val!(1)),
							}),
							Attr::Obj(AttrLeaf {
								key: Arc::from("b"),
								value: Box::new(log_val!(2)),
							}),
							Attr::Grp(AttrGroup {
								key: Arc::from("inner_group"),
								attrs: Arc::<[Attr]>::from(
									vec![Attr::Obj(AttrLeaf {
										key: Arc::from("z"),
										value: Box::new(log_val!(3)),
									})]
									.into_boxed_slice(),
								),
							}),
						]
						.into_boxed_slice(),
					),
				}),
			]
			.into_boxed_slice(),
		);
		assert_eq!(output, expected);
	}
}
