use std::{error::Error, io, io::Write, rc::Rc, sync::Arc, time::SystemTime};

use super::CommonHandler;
use crate::{Attr, Context, Handler, Level, Record, INFO};

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
}

impl<L: Level, C: Context, W: Write> Handler<L, C> for TextHandler<W> {
	type Error = io::Error;

	fn enabled(&self, _ctx: C, lvl: L) -> bool {
		lvl.value() >= self.0.min_level
	}

	fn handle(&mut self, ctx: C, rec: Record<L>) -> Result<(), Self::Error> {
		todo!()
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
