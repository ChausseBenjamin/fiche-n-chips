#[derive(Clone)]
pub struct Source {
	pub module: &'static str,
	pub file: &'static str,
	pub line: u32,
}

impl Source {
	pub fn new(module: &'static str, file: &'static str, line: u32) -> Self {
		Self { module, file, line }
	}
}

#[macro_export]
macro_rules! source {
	() => {
		$crate::Source::new(module_path!(), file!(), line!())
	};
}
