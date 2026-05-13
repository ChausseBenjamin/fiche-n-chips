use std::{
	collections::VecDeque, error::Error, io::Write, rc::Rc, sync::Arc,
	time::SystemTime,
};

use crate::{Attr, Context, Handler, Level, Record, INFO};

pub struct TextHandler<W: Write> {}

impl<W: Write> TextHandler<W> {
	pub fn new(writer: W) -> Self {
		Self {
			app_name: None,
			min_level: INFO.value(),
			attach_time: true,
			attach_source: false,
			attrs: Default::default(),
			writer,
		}
	}

	pub fn app_name(mut self, name: impl Into<Arc<str>>) -> Self {
		self.app_name = Some(name.into());
		self
	}

	pub fn min_level<L: Level>(mut self, level: L) -> Self {
		self.min_level = level.value();
		self
	}

	pub fn attach_source(mut self, enabled: bool) -> Self {
		self.attach_source = enabled;
		self
	}

	pub fn attach_time(mut self, enabled: bool) -> Self {
		self.attach_time = enabled;
		self
	}

	fn time_str(t: SystemTime) -> Rc<str> {
		todo!()
	}
}

impl<L: Level, C: Context, W: Write> Handler<L, C> for TextHandler<W> {
	type Error = crate::HandlerError;

	fn enabled(&self, ctx: C, lvl: L) -> bool {
		lvl.value() >= self.min_level
	}

	fn handle(&mut self, ctx: C, rec: Record<L>) -> Result<(), Error> {
		todo!()
	}

	fn with_attrs(&self, attrs: Arc<[Attr]>) -> Self {
		self.attrs.append(self.flatten_attrs(attrs));
		*self
	}

	fn with_group(&self, name: Arc<str>) -> Self {
		self.groups.push_back(name) * self
	}
}
