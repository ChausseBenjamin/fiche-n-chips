pub struct Source {
	module: &'static str,
	file: &'static str,
	line: u32,
}

impl Source {
	pub fn new(module: &'static str, file: &'static str, line: u32) -> Self {
		Self { module, file, line }
	}
}

#[macro_export]
macro_rules! source {
	() => {
		Source::new(module_path!(), file!(), line!())
	};
}
