use std::{any::Any, error::Error, io::Write, sync::Arc};

use super::CommonHandler;
use crate::{group, Attr, Handler, Level, Record};

pub struct TextHandler<W: Write>(CommonHandler<W>);

impl<W: Write> TextHandler<W> {
	pub fn new(writer: W) -> Self {
		Self(CommonHandler::new(writer))
	}

	pub fn app_name(mut self, name: impl Into<Arc<str>>) -> Self {
		self.0.app_name = Some(name.into());
		self
	}

	pub fn min_level(mut self, level: i8) -> Self {
		self.0.min_level = level;
		self
	}

	pub fn attach_source(mut self, enabled: bool) -> Self {
		self.0.attach_source = enabled;
		self
	}

	pub fn attach_time(mut self, enabled: bool) -> Self {
		self.0.attach_time = enabled;
		self
	}

	fn str_attrs(attrs: Vec<Attr>, prefix: Option<Arc<str>>) -> Vec<String> {
		let flatlist: Vec<String> = attrs
			.iter()
			.flat_map(|a| match a {
				Attr::Obj(leaf) => {
					vec![format!("{}=\"{}\"", leaf.key, leaf.value).to_string()]
				}
				Attr::Grp(group) => {
					Self::str_attrs(group.attrs.to_vec(), Some(group.key.clone()))
				}
			})
			.collect();
		if let Some(p) = prefix {
			flatlist.iter().map(|a| format!("{}.{}", p, a).to_string()).collect()
		} else {
			flatlist
		}
	}
}

impl<W: Write + Send + 'static> Handler for TextHandler<W> {
	fn enabled(&self, _ctx: &dyn Any, lvl: &dyn Level) -> bool {
		lvl.value() >= self.0.min_level
	}

	fn handle(
		&mut self,
		_ctx: &dyn Any,
		rec: Record,
	) -> Result<(), Box<dyn Error + Send + Sync>> {
		let mut attrs = self.0.attrs.clone();
		attrs.extend(rec.attrs);
		if self.0.attach_source {
			attrs.push(group!("source",
				"module" => rec.src.module,
				"file" => rec.src.file,
				"line" => rec.src.line,
			));
		}
		let rhs: String = Self::str_attrs(attrs, None).join(", ");
		let time = rec
			.time
			.duration_since(std::time::UNIX_EPOCH)
			.map(|d| d.as_secs().to_string())
			.unwrap_or_else(|_| "Invalid time".into());
		let lhs: String = match (self.0.app_name.clone(), self.0.attach_time) {
			(None, false) => format!("{} {}", rec.level, rec.msg),
			(None, true) => format!("{} {}| {}", rec.level, time, rec.msg),
			(Some(name), false) => format!("{} {}: {}", rec.level, name, rec.msg),
			(Some(name), true) => {
				format!("{} {}| {}: {}", rec.level, time, name, rec.msg)
			}
		};
		let mut writer = self.0.writer.lock().unwrap_or_else(|e| e.into_inner());
		writeln!(writer, "{}: {}", lhs, rhs)?;
		Ok(())
	}

	fn with_attrs(&mut self, attrs: Vec<Attr>) -> Box<dyn Handler> {
		Box::new(Self(self.0.with_attrs(attrs)))
	}

	fn with_group(&mut self, name: Arc<str>) -> Box<dyn Handler> {
		Box::new(Self(self.0.with_group(name)))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::sync::Arc;
	use std::time::SystemTime;
	use crate::{Record, Source, DEBUG, INFO, WARN, ERROR, attr, group};

	fn basic_record(msg: &str) -> Record {
		Record {
			time: SystemTime::UNIX_EPOCH,
			msg: Arc::from(msg),
			level: Arc::new(INFO),
			src: Source::new("test_module", "test_file.rs", 42),
			attrs: vec![],
		}
	}

	#[test]
	fn enabled_respects_min_level() {
		let handler = TextHandler::new(Vec::new()).min_level(WARN.value());
		assert!(!handler.enabled(&(), &DEBUG));
		assert!(!handler.enabled(&(), &INFO));
		assert!(handler.enabled(&(), &WARN));
		assert!(handler.enabled(&(), &ERROR));
	}

	#[test]
	fn handle_basic_with_time() {
		let buf = Vec::new();
		let mut handler = TextHandler::new(buf).min_level(DEBUG.value());
		let rec = basic_record("hello");
		handler.handle(&(), rec).unwrap();
		let writer = handler.0.writer.lock().unwrap();
		assert_eq!(
			String::from_utf8(writer.clone()).unwrap(),
			"INFO 0| hello: \n"
		);
	}

	#[test]
	fn handle_basic_without_time() {
		let buf = Vec::new();
		let mut handler =
			TextHandler::new(buf).attach_time(false).min_level(DEBUG.value());
		let rec = basic_record("hello");
		handler.handle(&(), rec).unwrap();
		let writer = handler.0.writer.lock().unwrap();
		assert_eq!(
			String::from_utf8(writer.clone()).unwrap(),
			"INFO hello: \n"
		);
	}

	#[test]
	fn handle_with_app_name() {
		let buf = Vec::new();
		let mut handler = TextHandler::new(buf)
			.app_name("myapp")
			.attach_time(false)
			.min_level(DEBUG.value());
		let rec = basic_record("hello");
		handler.handle(&(), rec).unwrap();
		let writer = handler.0.writer.lock().unwrap();
		assert_eq!(
			String::from_utf8(writer.clone()).unwrap(),
			"INFO myapp: hello: \n"
		);
	}

	#[test]
	fn handle_with_app_name_and_time() {
		let buf = Vec::new();
		let mut handler = TextHandler::new(buf)
			.app_name("myapp")
			.min_level(DEBUG.value());
		let rec = basic_record("hello");
		handler.handle(&(), rec).unwrap();
		let writer = handler.0.writer.lock().unwrap();
		assert_eq!(
			String::from_utf8(writer.clone()).unwrap(),
			"INFO 0| myapp: hello: \n"
		);
	}

	#[test]
	fn handle_with_source() {
		let buf = Vec::new();
		let mut handler = TextHandler::new(buf)
			.attach_source(true)
			.attach_time(false)
			.min_level(DEBUG.value());
		let rec = basic_record("hello");
		handler.handle(&(), rec).unwrap();
		let writer = handler.0.writer.lock().unwrap();
		assert_eq!(
			String::from_utf8(writer.clone()).unwrap(),
			"INFO hello: source.module=\"test_module\", source.file=\"test_file.rs\", source.line=\"42\"\n"
		);
	}

	#[test]
	fn handle_with_source_and_time() {
		let buf = Vec::new();
		let mut handler = TextHandler::new(buf)
			.attach_source(true)
			.min_level(DEBUG.value());
		let rec = basic_record("hello");
		handler.handle(&(), rec).unwrap();
		let writer = handler.0.writer.lock().unwrap();
		assert_eq!(
			String::from_utf8(writer.clone()).unwrap(),
			"INFO 0| hello: source.module=\"test_module\", source.file=\"test_file.rs\", source.line=\"42\"\n"
		);
	}

	#[test]
	fn handle_with_handler_attrs() {
		let buf = Vec::new();
		let mut handler = TextHandler::new(buf)
			.attach_time(false)
			.min_level(DEBUG.value());
		handler.0.attrs = vec![
			attr!("method", "GET"),
			attr!("status", 200),
		];
		let rec = basic_record("request");
		handler.handle(&(), rec).unwrap();
		let writer = handler.0.writer.lock().unwrap();
		assert_eq!(
			String::from_utf8(writer.clone()).unwrap(),
			"INFO request: method=\"GET\", status=\"200\"\n"
		);
	}

	#[test]
	fn handle_with_grouped_handler_attrs() {
		let buf = Vec::new();
		let mut handler = TextHandler::new(buf)
			.attach_time(false)
			.min_level(DEBUG.value());
		handler.0.attrs = vec![group!("http",
			"method" => "GET",
			"status" => 200,
		)];
		let rec = basic_record("request");
		handler.handle(&(), rec).unwrap();
		let writer = handler.0.writer.lock().unwrap();
		assert_eq!(
			String::from_utf8(writer.clone()).unwrap(),
			"INFO request: http.method=\"GET\", http.status=\"200\"\n"
		);
	}

	#[test]
	fn handle_writes_regardless_of_min_level() {
		// `handle` does not filter; filtering is the caller's
		// responsibility via `enabled`. This test documents that contract.
		let buf = Vec::new();
		let mut handler = TextHandler::new(buf)
			.attach_time(false)
			.min_level(ERROR.value());
		let rec = Record {
			time: SystemTime::UNIX_EPOCH,
			msg: Arc::from("below_min_level"),
			level: Arc::new(WARN),
			src: Source::new("m", "f", 1),
			attrs: vec![],
		};
		handler.handle(&(), rec).unwrap();
		let writer = handler.0.writer.lock().unwrap();
		assert!(!writer.is_empty());
		assert_eq!(
			String::from_utf8(writer.clone()).unwrap(),
			"WARN below_min_level: \n"
		);
	}

	#[test]
	fn handle_with_different_levels() {
		let buf = Vec::new();
		let mut handler = TextHandler::new(buf).min_level(DEBUG.value());

		let rec = Record {
			time: SystemTime::UNIX_EPOCH,
			msg: Arc::from("debug msg"),
			level: Arc::new(DEBUG),
			src: Source::new("m", "f", 1),
			attrs: vec![],
		};
		handler.handle(&(), rec).unwrap();

		let rec = Record {
			time: SystemTime::UNIX_EPOCH,
			msg: Arc::from("info msg"),
			level: Arc::new(INFO),
			src: Source::new("m", "f", 1),
			attrs: vec![],
		};
		handler.handle(&(), rec).unwrap();

		let writer = handler.0.writer.lock().unwrap();
		let output = String::from_utf8(writer.clone()).unwrap();
		assert!(output.contains("DEBUG 0| debug msg: \n"));
		assert!(output.contains("INFO 0| info msg: \n"));
	}

	#[test]
	fn with_attrs_adds_to_handler_attrs() {
		let buf = Vec::new();
		let mut handler = TextHandler::new(buf)
			.attach_time(false)
			.min_level(DEBUG.value());
		let mut forked =
			handler.with_attrs(vec![attr!("fork", true)]);

		let rec1 = basic_record("orig");
		handler.handle(&(), rec1).unwrap();

		let rec2 = basic_record("forked");
		forked.handle(&(), rec2).unwrap();

		let writer = handler.0.writer.lock().unwrap();
		let output = String::from_utf8(writer.clone()).unwrap();
		let lines: Vec<&str> = output.lines().collect();
		assert_eq!(lines.len(), 2);
		assert_eq!(lines[0], "INFO orig: ");
		assert_eq!(lines[1], "INFO forked: fork=\"true\"");
	}

	#[test]
	fn with_group_nests_attrs() {
		let buf = Vec::new();
		let mut handler = TextHandler::new(buf)
			.attach_time(false)
			.min_level(DEBUG.value());
		handler.0.attrs = vec![attr!("key", "val")];
		let mut grouped = handler.with_group("ns".into());

		let rec = basic_record("test");
		grouped.handle(&(), rec).unwrap();

		let writer = handler.0.writer.lock().unwrap();
		assert_eq!(
			String::from_utf8(writer.clone()).unwrap(),
			"INFO test: ns.key=\"val\"\n"
		);
	}

	#[test]
	fn str_attrs_flat() {
		let attrs = vec![attr!("a", 1), attr!("b", "two")];
		let result = TextHandler::<Vec<u8>>::str_attrs(attrs, None);
		assert_eq!(result, vec!["a=\"1\"", "b=\"two\""]);
	}

	#[test]
	fn str_attrs_with_prefix() {
		let attrs = vec![attr!("x", 10)];
		let result =
			TextHandler::<Vec<u8>>::str_attrs(attrs, Some("root".into()));
		assert_eq!(result, vec!["root.x=\"10\""]);
	}

	#[test]
	fn str_attrs_nested_group() {
		let attrs = vec![group!("outer", "inner" => "value")];
		let result = TextHandler::<Vec<u8>>::str_attrs(attrs, None);
		assert_eq!(result, vec!["outer.inner=\"value\""]);
	}

	#[test]
	fn str_attrs_empty() {
		let result = TextHandler::<Vec<u8>>::str_attrs(vec![], None);
		assert!(result.is_empty());
	}

	#[test]
	fn str_attrs_deeply_nested() {
		let attrs = vec![group!("a",
			"b" => "1",
			group!("d",
				"e" => "2",
			),
		)];
		let result = TextHandler::<Vec<u8>>::str_attrs(attrs, None);
		assert_eq!(
			result,
			vec!["a.b=\"1\"", "a.d.e=\"2\""]
		);
	}
}
