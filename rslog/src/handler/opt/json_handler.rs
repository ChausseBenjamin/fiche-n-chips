use std::{any::Any, error::Error, io::Write, sync::Arc};

use serde_json::{Map, Value};

use super::super::CommonHandler;
use crate::{Attr, Handler, Level, Record};

pub struct JsonHandler<W: Write>(CommonHandler<W>);

impl<W: Write> JsonHandler<W> {
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

	fn attrs_to_map(attrs: &[Attr]) -> Map<String, Value> {
		let mut map = Map::new();
		for attr in attrs {
			match attr {
				Attr::Obj(leaf) => {
					map.insert(leaf.key.to_string(), Value::String(leaf.value.to_string()));
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
			let timestamp = rec
				.time
				.duration_since(std::time::UNIX_EPOCH)
				.map(|d| d.as_secs())
				.unwrap_or(0);
			obj.insert("time".into(), Value::Number(timestamp.into()));
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
