use std::any::Any;
use std::error::Error;
use std::io::Write;
use std::sync::Arc;

use serde_json::{Map, Value};

use super::super::CommonHandler;
use crate::{Attr, Handler, Level, Record};

pub struct JsonHandler<W: Write>(CommonHandler<W>);

impl<W: Write> JsonHandler<W> {
	pub fn new(writer: W) -> Self { Self(CommonHandler::new(writer)) }

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

	fn attrs_to_map(attrs: &[Attr]) -> Map<String, Value> {
		let mut map = Map::new();
		for attr in attrs {
			match attr {
				Attr::Obj(leaf) => {
					map.insert(
						leaf.key.to_string(),
						Value::String(leaf.value.to_string()),
					);
				}
				Attr::Grp(group) => {
					map.insert(
						group.key.to_string(),
						Value::Object(Self::attrs_to_map(&group.attrs)),
					);
				}
			}
		}
		map
	}
}

impl<W: Write + Send + 'static> Handler for JsonHandler<W> {
	fn enabled(&self, _ctx: &dyn Any, lvl: &dyn Level) -> bool {
		lvl.value() >= self.0.min_level
	}

	fn handle(
		&mut self,
		_ctx: &dyn Any,
		rec: Record,
	) -> Result<(), Box<dyn Error + Send + Sync>> {
		let mut obj = Map::new();

		obj.insert("msg".into(), Value::String(rec.msg.to_string()));
		obj.insert("level".into(), Value::String(rec.level.to_string()));

		if self.0.attach_time {
			obj.insert(
				"time".into(),
				Value::String(crate::util::format_time(rec.time)),
			);
		}

		if let Some(name) = &self.0.app_name {
			obj.insert("app_name".into(), Value::String(name.to_string()));
		}

		if self.0.attach_source {
			let mut src = Map::new();
			src.insert("module".into(), Value::String(rec.src.module.into()));
			src.insert("file".into(), Value::String(rec.src.file.into()));
			src.insert("line".into(), Value::Number(rec.src.line.into()));
			obj.insert("source".into(), Value::Object(src));
		}

		let attr_map = Self::attrs_to_map(&self.0.attrs);
		for (k, v) in attr_map {
			obj.insert(k, v);
		}

		let rec_attr_map = Self::attrs_to_map(&rec.attrs);
		for (k, v) in rec_attr_map {
			obj.insert(k, v);
		}

		let mut writer = self.0.writer.lock().unwrap_or_else(|e| e.into_inner());
		writeln!(writer, "{}", Value::Object(obj))?;
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
	use std::sync::Arc;
	use std::time::SystemTime;

	use serde_json::Value;

	use super::*;
	use crate::{attr, group, Record, Source, DEBUG, ERROR, INFO, WARN};

	fn basic_record(msg: &str) -> Record {
		Record {
			time:  SystemTime::UNIX_EPOCH,
			msg:   Arc::from(msg),
			level: Arc::new(INFO),
			src:   Source::new("test_module", "test_file.rs", 42),
			attrs: vec![],
		}
	}

	#[test]
	fn enabled_respects_min_level() {
		let handler = JsonHandler::new(Vec::new()).min_level(WARN.value());
		assert!(!handler.enabled(&(), &DEBUG));
		assert!(!handler.enabled(&(), &INFO));
		assert!(handler.enabled(&(), &WARN));
		assert!(handler.enabled(&(), &ERROR));
	}

	#[test]
	fn handle_basic() {
		let buf = Vec::new();
		let mut handler =
			JsonHandler::new(buf).attach_time(false).min_level(DEBUG.value());
		let rec = basic_record("hello");
		handler.handle(&(), rec).unwrap();
		let writer = handler.0.writer.lock().unwrap();
		let output: Value = serde_json::from_slice(&writer).unwrap();
		assert_eq!(output["msg"], "hello");
		assert_eq!(output["level"], "INFO");
		assert!(!output.as_object().unwrap().contains_key("time"));
		assert!(!output.as_object().unwrap().contains_key("app_name"));
		assert!(!output.as_object().unwrap().contains_key("source"));
	}

	#[test]
	fn handle_with_time() {
		let buf = Vec::new();
		let mut handler = JsonHandler::new(buf).min_level(DEBUG.value());
		let rec = basic_record("hello");
		handler.handle(&(), rec).unwrap();
		let writer = handler.0.writer.lock().unwrap();
		let output: Value = serde_json::from_slice(&writer).unwrap();
		assert!(output["time"].is_string(), "time is not a string: {:?}", output["time"]);
		let time_str = output["time"].as_str().unwrap();
		assert_eq!(time_str.len(), 23, "time string length: {time_str}");
	}

	#[test]
	fn handle_with_app_name() {
		let buf = Vec::new();
		let mut handler = JsonHandler::new(buf)
			.app_name("myapp")
			.attach_time(false)
			.min_level(DEBUG.value());
		let rec = basic_record("hello");
		handler.handle(&(), rec).unwrap();
		let writer = handler.0.writer.lock().unwrap();
		let output: Value = serde_json::from_slice(&writer).unwrap();
		assert_eq!(output["app_name"], "myapp");
	}

	#[test]
	fn handle_with_app_name_and_time() {
		let buf = Vec::new();
		let mut handler =
			JsonHandler::new(buf).app_name("myapp").min_level(DEBUG.value());
		let rec = basic_record("hello");
		handler.handle(&(), rec).unwrap();
		let writer = handler.0.writer.lock().unwrap();
		let output: Value = serde_json::from_slice(&writer).unwrap();
		assert_eq!(output["app_name"], "myapp");
		assert!(output["time"].is_string(), "time is not a string: {:?}", output["time"]);
	}

	#[test]
	fn handle_with_source() {
		let buf = Vec::new();
		let mut handler = JsonHandler::new(buf)
			.attach_source(true)
			.attach_time(false)
			.min_level(DEBUG.value());
		let rec = basic_record("hello");
		handler.handle(&(), rec).unwrap();
		let writer = handler.0.writer.lock().unwrap();
		let output: Value = serde_json::from_slice(&writer).unwrap();
		assert_eq!(output["source"]["module"], "test_module");
		assert_eq!(output["source"]["file"], "test_file.rs");
		assert_eq!(output["source"]["line"], 42);
	}

	#[test]
	fn handle_with_handler_attrs() {
		let buf = Vec::new();
		let mut handler =
			JsonHandler::new(buf).attach_time(false).min_level(DEBUG.value());
		handler.0.attrs = vec![attr!("method", "GET"), attr!("status", 200)];
		let rec = basic_record("request");
		handler.handle(&(), rec).unwrap();
		let writer = handler.0.writer.lock().unwrap();
		let output: Value = serde_json::from_slice(&writer).unwrap();
		assert_eq!(output["method"], "GET");
		assert_eq!(output["status"], "200");
	}

	#[test]
	fn handle_with_grouped_handler_attrs() {
		let buf = Vec::new();
		let mut handler =
			JsonHandler::new(buf).attach_time(false).min_level(DEBUG.value());
		handler.0.attrs = vec![group!("http",
			"method" => "GET",
			"status" => 200,
		)];
		let rec = basic_record("request");
		handler.handle(&(), rec).unwrap();
		let writer = handler.0.writer.lock().unwrap();
		let output: Value = serde_json::from_slice(&writer).unwrap();
		assert_eq!(output["http"]["method"], "GET");
		assert_eq!(output["http"]["status"], "200");
	}

	#[test]
	fn handle_with_all_options() {
		let buf = Vec::new();
		let mut handler = JsonHandler::new(buf)
			.app_name("myapp")
			.attach_source(true)
			.min_level(DEBUG.value());
		handler.0.attrs = vec![group!("http",
			"method" => "POST",
		)];
		let rec = basic_record("request");
		handler.handle(&(), rec).unwrap();
		let writer = handler.0.writer.lock().unwrap();
		let output: Value = serde_json::from_slice(&writer).unwrap();
		assert_eq!(output["msg"], "request");
		assert_eq!(output["level"], "INFO");
		assert_eq!(output["app_name"], "myapp");
		assert!(output["time"].is_string(), "time is not a string: {:?}", output["time"]);
		assert_eq!(output["source"]["module"], "test_module");
		assert_eq!(output["http"]["method"], "POST");
	}

	#[test]
	fn handle_writes_regardless_of_min_level() {
		// `handle` does not filter; filtering is the caller's
		// responsibility via `enabled`. This test documents that contract.
		let buf = Vec::new();
		let mut handler =
			JsonHandler::new(buf).attach_time(false).min_level(ERROR.value());
		let rec = Record {
			time:  SystemTime::UNIX_EPOCH,
			msg:   Arc::from("warn_msg"),
			level: Arc::new(WARN),
			src:   Source::new("m", "f", 1),
			attrs: vec![],
		};
		handler.handle(&(), rec).unwrap();
		let writer = handler.0.writer.lock().unwrap();
		assert!(!writer.is_empty());
		let output: Value = serde_json::from_slice(&writer).unwrap();
		assert_eq!(output["msg"], "warn_msg");
		assert_eq!(output["level"], "WARN");
	}

	#[test]
	fn with_attrs_adds_to_handler_attrs() {
		let buf = Vec::new();
		let mut handler =
			JsonHandler::new(buf).attach_time(false).min_level(DEBUG.value());
		let mut forked = handler.with_attrs(vec![attr!("env", "prod")]);

		let rec1 = basic_record("orig");
		handler.handle(&(), rec1).unwrap();

		let rec2 = basic_record("forked");
		forked.handle(&(), rec2).unwrap();

		let writer = handler.0.writer.lock().unwrap();
		let output = String::from_utf8(writer.clone()).unwrap();
		let lines: Vec<&str> = output.lines().collect();
		assert_eq!(lines.len(), 2);

		let first: Value = serde_json::from_str(lines[0]).unwrap();
		assert_eq!(first["msg"], "orig");
		assert!(!first.as_object().unwrap().contains_key("env"));

		let second: Value = serde_json::from_str(lines[1]).unwrap();
		assert_eq!(second["msg"], "forked");
		assert_eq!(second["env"], "prod");
	}

	#[test]
	fn with_group_nests_attrs() {
		let buf = Vec::new();
		let mut handler =
			JsonHandler::new(buf).attach_time(false).min_level(DEBUG.value());
		handler.0.attrs = vec![attr!("key", "val")];
		let mut grouped = handler.with_group("ns".into());

		let rec = basic_record("test");
		grouped.handle(&(), rec).unwrap();

		let writer = handler.0.writer.lock().unwrap();
		let output: Value = serde_json::from_slice(&writer).unwrap();
		assert_eq!(output["ns"]["key"], "val");
	}

	#[test]
	fn attrs_to_map_flat() {
		let attrs = vec![attr!("a", 1), attr!("b", "two")];
		let map = JsonHandler::<Vec<u8>>::attrs_to_map(&attrs);
		assert_eq!(map["a"], "1");
		assert_eq!(map["b"], "two");
	}

	#[test]
	fn attrs_to_map_grouped() {
		let attrs = vec![group!("http",
			"method" => "GET",
			"status" => 200,
		)];
		let map = JsonHandler::<Vec<u8>>::attrs_to_map(&attrs);
		assert_eq!(map["http"]["method"], "GET");
		assert_eq!(map["http"]["status"], "200");
	}

	#[test]
	fn attrs_to_map_empty() {
		let map = JsonHandler::<Vec<u8>>::attrs_to_map(&[]);
		assert!(map.is_empty());
	}

	#[test]
	fn attrs_to_map_nested() {
		let attrs = vec![group!("a",
			"b" => "1",
			group!("d",
				"e" => "2",
			),
		)];
		let map = JsonHandler::<Vec<u8>>::attrs_to_map(&attrs);
		assert_eq!(map["a"]["b"], "1");
		assert_eq!(map["a"]["d"]["e"], "2");
	}
}
