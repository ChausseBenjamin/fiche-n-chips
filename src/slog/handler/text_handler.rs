use std::{error::Error, io::Write, sync::Arc};

use crate::{Attr, Context, Handler, HandlerError, Level, Record, INFO};

pub struct TextHandler<W: Write> {
	// can be useful in for sorting if multiple apps log in the same stream
	app_name: Option<Arc<str>>,
	min_level: i8,
	attach_time: bool,
	attach_source: bool,
	writer: W,

	attrs: Arc<str>, // not Attr -> pre-computed as a string ahead of time
}

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

	pub fn flatten_attrs(
		attrs: Arc<[Attr]>,
		prefix: Option<&str>,
	) -> Result<Arc<str>, HandlerError> {
		// Helper to join a list of pieces with ", " and return String
		fn join_pieces<I>(iter: I) -> String
		where
			I: IntoIterator<Item = String>,
		{
			iter.into_iter().collect::<Vec<_>>().join(", ")
		}

		// Build top-level pieces by mapping each Attr to a String (recursing for
		// groups).
		let pieces = attrs.iter().map(|a| match a {
			Attr::Obj(leaf) => {
				// key (with optional prefix)
				let key = if let Some(p) = prefix {
					format!("{}.{}", p, leaf.key)
				} else {
					leaf.key.to_string()
				};
				format!("{}=\"{}\"", key, leaf.value);
			}
			Attr::Grp(group) => {
				// new prefix: either prefix + "." + group.key or just group.key
				let new_prefix = if let Some(p) = prefix {
					format!("{}.{}", p, group.key)
				} else {
					group.key.to_string()
				};
				// Recurse to format group's attrs with new_prefix
				self.flatten_attrs(group.attrs.clone(), Some(&new_prefix)).to_string()
			}
		});

		Ok(Arc::from(join_pieces(pieces)))
	}
}

impl<L: Level, C: Context, W: Write> Handler<L, C> for TextHandler<W> {
	type Error = crate::HandlerError;

	fn enabled(&self, ctx: C, lvl: L) -> bool {
		lvl.value() >= self.min_level
	}

	fn handle(&self, ctx: C, rec: Record<L>) -> Result<(), Self::Error> {
		todo!()
	}

	fn with_attrs(&self, attrs: Arc<[Attr]>) -> Self {
		todo!()
	}

	fn with_group(&self, name: Arc<str>) -> Self {
		todo!()
	}
}
