#![feature(min_specialization)]
use std::fmt::Display;

pub trait LogValue {
	fn log_value(&self) -> String;
}

impl<T> LogValue for T
where
	T: Sync + Clone + Display,
{
	fn log_value(&self) -> String {
		self.to_string()
	}
}

// impl<T: Sync + Clone + Display> LogValue for T {
// 	fn log_value(&self) -> String {
// 		self.to_string()
// 	}
// }

#[cfg(test)]
mod tests {
	use std::fmt;

	use super::*;

	/// Only implements Display to ensure the default implementation
	/// is used by default
	#[derive(Clone)]
	struct VanillaType {
		value: usize,
	}
	impl fmt::Display for VanillaType {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			write!(f, "MyType({})", self.value)
		}
	}

	/// Implements both Display and LogValue to see if priority goes to
	/// the LogValue type
	#[derive(Clone)]
	struct CustomType {
		value: usize,
	}
	impl fmt::Display for CustomType {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			write!(f, "MyType({})", self.value)
		}
	}
	impl LogValue for CustomType {
		fn log_value(&self) -> String {
			return "CUSTOM".to_string();
		}
	}

	#[tokio::test]
	async fn generic_datatypes() {
		let usize_v: usize = 123;
		assert_eq!(usize_v.log_value(), "123");

		let isize_v: isize = -456;
		assert_eq!(isize_v.log_value(), "-456");

		let i32_v: i32 = 42;
		assert_eq!(i32_v.log_value(), "42");

		let bool_v = true;
		assert_eq!(bool_v.log_value(), "true");

		let string_v = String::from("hello");
		assert_eq!(string_v.log_value(), "hello");

		let str_v = "world";
		assert_eq!(str_v.log_value(), "world");

		let float_v: f64 = 3.14;
		assert_eq!(float_v.log_value(), "3.14");
	}

	#[tokio::test]
	async fn custom_vanilla_types() {
		let input = VanillaType { value: 42 };
		assert_eq!(input.log_value(), "MyType(42)");
	}

	#[tokio::test]
	async fn custom_complex_types() {
		let input = CustomType { value: 42 };
		assert_eq!(input.log_value(), "CUSTOM");
	}
}
