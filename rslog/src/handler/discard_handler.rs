use std::any::Any;
use std::error::Error;
use std::sync::Arc;

use crate::{Attr, Handler, Level, Record};

pub struct DiscardHandler;

impl Handler for DiscardHandler {
	fn enabled(&self, _ctx: &dyn Any, _lvl: &dyn Level) -> bool { false }

	fn handle(
		&mut self,
		_ctx: &dyn Any,
		_rec: Record,
	) -> Result<(), Box<dyn Error + Send + Sync>> {
		Ok(())
	}

	fn with_attrs(&mut self, _attrs: Vec<Attr>) -> Box<dyn Handler> {
		Box::new(Self)
	}

	fn with_group(&mut self, _name: Arc<str>) -> Box<dyn Handler> {
		Box::new(Self)
	}
}
