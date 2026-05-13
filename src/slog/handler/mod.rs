use std::sync::Arc;

mod common_handler;
mod text_handler;

mod opt {
	#[cfg(feature = "json")]
	mod json_handler;

	#[cfg(feature = "json")]
	pub use self::json_handler::*;
}

pub use self::text_handler::*;
use self::{common_handler::*, opt::*};
use crate::{attr::Attr, context::Context, level::Level, record::Record};

pub trait Handler<L: Level, C: Context> {
	type Error;
	fn enabled(&self, ctx: C, lvl: L) -> bool;
	fn handle(&self, ctx: C, rec: Record<L>) -> Result<(), Self::Error>;
	fn with_attrs(&self, attrs: Arc<[Attr]>) -> Self;
	fn with_group(&self, name: Arc<str>) -> Self;
}
