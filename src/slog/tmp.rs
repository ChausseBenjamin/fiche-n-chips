pub trait Foo {
	fn do_foo(&self) -> usize;
}

pub trait Bar {
	fn do_bar(&self) -> usize;
}

#[derive(Debug, PartialEq)]
pub enum FooBar<F: Foo, B: Bar> {
	IsFoo(F),
	IsBar(B),
}

impl<F: Foo, B: Bar> FooBar<F, B> {
	fn do_resolved(&self) -> usize {
		match self {
			FooBar::IsFoo(f) => f.do_foo(),
			FooBar::IsBar(b) => b.do_bar(),
		}
	}
}

#[doc(hidden)]
pub mod __prefer_foo_impl {
	use super::{Bar, Foo, FooBar};

	#[derive(Debug, PartialEq)]
	pub enum Never {}
	impl Foo for Never {
		fn do_foo(&self) -> usize {
			match *self {}
		}
	}
	impl Bar for Never {
		fn do_bar(&self) -> usize {
			match *self {}
		}
	}

	pub struct Wrap<T>(pub T);

	// Inherent impl -> wins over trait impl when T: Foo.
	// Returns FooBar<T, Never>: The Bar slot is dead.
	impl<T: Foo> Wrap<T> {
		pub fn resolve(self) -> FooBar<T, Never> {
			FooBar::IsFoo(self.0)
		}
	}

	// Fallback trait -> only reached when T: !Foo.
	// Returns FooBar<Never, T>: The Foo slot is dead.
	pub trait ResolveFallback {
		type Inner: Bar;
		fn resolve(self) -> FooBar<Never, Self::Inner>;
	}

	impl<T: Bar> ResolveFallback for Wrap<T> {
		type Inner = T;
		fn resolve(self) -> FooBar<Never, T> {
			FooBar::IsBar(self.0)
		}
	}
}

#[macro_export]
macro_rules! prefer_foo {
	($val:expr) => {{
		// Bring the fallback trait into scope without polluting the caller's
		// namespace. Inherent methods shadow this automatically when T: Foo.
		use $crate::tmp::__prefer_foo_impl::ResolveFallback as _;
		$crate::tmp::__prefer_foo_impl::Wrap($val).resolve()
	}};
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Debug, PartialEq)]
	pub struct OnlyFoo;
	impl Foo for OnlyFoo {
		fn do_foo(&self) -> usize {
			1
		}
	}

	#[derive(Debug, PartialEq)]
	pub struct OnlyBar;
	impl Bar for OnlyBar {
		fn do_bar(&self) -> usize {
			2
		}
	}

	#[derive(Debug, PartialEq)]
	pub struct HasBoth;
	impl Foo for HasBoth {
		fn do_foo(&self) -> usize {
			3
		}
	}
	impl Bar for HasBoth {
		fn do_bar(&self) -> usize {
			4
		}
	}

	#[test]
	fn only_has_foo() {
		let input = OnlyFoo;
		let output = prefer_foo!(input);
		let expected = FooBar::IsFoo(OnlyFoo);
		assert_eq!(output, expected);
		assert_eq!(output.do_resolved(), 1)
	}

	#[test]
	fn only_has_bar() {
		let input = OnlyBar;
		let output = prefer_foo!(input);
		let expected = FooBar::IsBar(OnlyBar);
		assert_eq!(output, expected);
		assert_eq!(output.do_resolved(), 2)
	}

	#[test]
	fn choose_foo_when_both() {
		let input = HasBoth;
		let output = prefer_foo!(input);
		let expected = FooBar::IsFoo(HasBoth);
		assert_eq!(output, expected);
		assert_eq!(output.do_resolved(), 3)
	}
}
