use std::{io::Write, sync::Arc};

use crate::{group, Attr, AttrGroup, Level, INFO};

// keys for built-in attributes
pub const TIME_KEY: &str = "time";
pub const LEVEL_KEY: &str = "level";
pub const MESSAGE_KEY: &str = "msg";
pub const SOURCE_KEY: &str = "source";

// pub fn ReplaceAttr(groups: &[&str],attr: Attr) -> Attr;

pub(super) struct CommonHandler<W: Write> {
	// can be useful in for sorting if multiple apps log in the same stream
	pub app_name: Option<Arc<str>>,
	pub min_level: i8,
	pub attach_time: bool,
	pub attach_source: bool,
	// XXX: might need a mutex if writers shit the bed with concurent calls
	pub writer: Arc<W>,

	// This is a low-effort implementation leaving lots of performance on the
	// table. Pre-computing attrs belonging to a handler as ready-made string
	// shards could greatly diminish the compute cost of every log statement
	// that uses it.
	// I want something that works. Optimizing performance comes later as this
	// is just an implementation detail which isn't user-facing/accessible.
	attrs: Vec<Attr>,
}

impl<W: Write> Clone for CommonHandler<W> {
	fn clone(&self) -> Self {
		Self {
			app_name: self.app_name.clone(),
			min_level: self.min_level.clone(),
			attach_time: self.attach_time.clone(),
			attach_source: self.attach_source.clone(),
			writer: self.writer.clone(),
			attrs: self.attrs.clone(),
		}
	}
}

impl<W: Write> CommonHandler<W> {
	pub fn new(writer: W) -> Self {
		Self {
			app_name: None,
			min_level: INFO.value(),
			attach_time: true,
			attach_source: false,
			attrs: Default::default(),
			writer: Arc::new(writer),
		}
	}

	pub fn with_attrs(&self, attrs: Vec<Attr>) -> Self {
		todo!()
	}

	pub fn with_group(&self, name: Arc<str>) -> Self {
		let group = Attr::Grp(AttrGroup {
			key: name,
			attrs: self.attrs.clone().into(),
		});
		Self {
			app_name: self.app_name.clone(),
			min_level: self.min_level,
			attach_time: self.attach_time,
			attach_source: self.attach_source,
			writer: self.writer.clone(),
			attrs: vec![group],
		}
	}
}
