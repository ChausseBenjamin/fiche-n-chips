use std::{any::Any, error::Error, sync::Arc};

use crate::{Attr, Handler, Level, Record};

pub struct MuxHandler(Box<[Box<dyn Handler>]>);

impl MuxHandler {
	pub fn new(handlers: Box<[Box<dyn Handler>]>) -> Self {
		Self(handlers)
	}
}

impl Handler for MuxHandler {
	fn enabled(&self, ctx: &dyn Any, lvl: &dyn Level) -> bool {
		self.0.iter().any(|h| h.enabled(ctx, lvl))
	}

	fn handle(
		&mut self,
		ctx: &dyn Any,
		rec: Record,
	) -> Result<(), Box<dyn Error + Send + Sync>> {
		let mut first_err = None;
		for h in self.0.iter_mut() {
			if let Err(e) = h.handle(ctx, rec.clone()) {
				if first_err.is_none() {
					first_err = Some(e);
				}
			}
		}
		match first_err {
			Some(e) => Err(e),
			None => Ok(()),
		}
	}

	fn with_attrs(&mut self, attrs: Vec<Attr>) -> Box<dyn Handler> {
		let children: Vec<Box<dyn Handler>> = self
			.0
			.iter_mut()
			.map(|h| h.with_attrs(attrs.clone()))
			.collect();
		Box::new(MuxHandler(children.into_boxed_slice()))
	}

	fn with_group(&mut self, name: Arc<str>) -> Box<dyn Handler> {
		let children: Vec<Box<dyn Handler>> = self
			.0
			.iter_mut()
			.map(|h| h.with_group(name.clone()))
			.collect();
		Box::new(MuxHandler(children.into_boxed_slice()))
	}
}
