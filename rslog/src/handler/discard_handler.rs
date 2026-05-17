use std::{convert::Infallible, sync::Arc};

use crate::{Attr, Context, Handler, Level, Record};

pub struct DiscardHandler;

impl Handler for DiscardHandler {
	type Error = Infallible;

	fn enabled<C: Context>(&self, _ctx: &C, _lvl: &dyn Level) -> bool {
		false
	}

	fn handle<C: Context>(
		&mut self,
		_ctx: &C,
		_rec: Record,
	) -> Result<(), Self::Error> {
		Ok(())
	}

	fn with_attrs(&mut self, _attrs: Vec<Attr>) -> Self {
		DiscardHandler
	}

	fn with_group(&mut self, _name: Arc<str>) -> Self {
		DiscardHandler
	}
}
