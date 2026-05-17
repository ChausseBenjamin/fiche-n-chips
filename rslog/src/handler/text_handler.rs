use std::{io, io::Write, sync::Arc};

use super::CommonHandler;
use crate::{group, Attr, Context, Handler, Level, Record};

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

impl<W: Write> Handler for TextHandler<W> {
	type Error = io::Error;

	fn enabled<C: Context>(&self, _ctx: &C, lvl: &dyn Level) -> bool {
		lvl.value() >= self.0.min_level
	}

	fn handle<C: Context>(
		&mut self,
		_ctx: &C,
		rec: Record,
	) -> Result<(), Self::Error> {
		let mut attrs = self.0.attrs.clone();
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
		writeln!(writer, "{}: {}", lhs, rhs)
	}

	fn with_attrs(&mut self, attrs: Vec<Attr>) -> Self {
		Self(self.0.with_attrs(attrs))
	}

	fn with_group(&mut self, name: Arc<str>) -> Self {
		Self(self.0.with_group(name))
	}
}
