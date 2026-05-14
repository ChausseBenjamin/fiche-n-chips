use std::{io, io::Write, sync::Arc};

use super::CommonHandler;
use crate::{Attr, Context, Handler, Level, Record};

const TIME_NOT_DONE: &str = "Unimplemented: time in logs";

pub struct TextHandler<W: Write>(CommonHandler<W>);

impl<W: Write> TextHandler<W> {
	pub fn new(writer: W) -> Self {
		Self {
			0: CommonHandler::new(writer),
		}
	}

	pub fn app_name(mut self, name: impl Into<Arc<str>>) -> Self {
		self.0.app_name = Some(name.into());
		self
	}

	pub fn min_level<L: Level>(mut self, level: L) -> Self {
		self.0.min_level = level.value();
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
					// TODO: escape strings and shit?
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

impl<L: Level, C: Context, W: Write> Handler<L, C> for TextHandler<W> {
	type Error = io::Error;

	fn enabled(&self, _ctx: C, lvl: L) -> bool {
		lvl.value() >= self.0.min_level
	}

	fn handle(&mut self, _ctx: C, rec: Record<L>) -> Result<(), Self::Error> {
		// TODO: Format time in the finishe version
		let lvl = rec.level.disp();
		let rhs: String = Self::str_attrs(self.0.attrs.clone(), None).join(", ");
		let lhs: String = match (self.0.app_name.clone(), self.0.attach_time) {
			(None, false) => format!("{} {}", lvl, rec.msg),
			(None, true) => format!("{} {}| {}", lvl, TIME_NOT_DONE, rec.msg),
			(Some(name), false) => format!("{} {}: {}", lvl, name, rec.msg),
			(Some(name), true) => {
				format!("{} {}| {}: {}", lvl, TIME_NOT_DONE, name, rec.msg)
			}
		};
		let mut writer = self.0.writer.lock().unwrap_or_else(|e| e.into_inner());
		writeln!(writer, "{}: {}", lhs, rhs)
	}

	fn with_attrs(&mut self, attrs: Vec<Attr>) -> Self {
		Self {
			0: self.0.with_attrs(attrs),
		}
	}

	fn with_group(&mut self, name: Arc<str>) -> Self {
		Self {
			0: self.0.with_group(name),
		}
	}
}
