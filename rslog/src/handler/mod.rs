use std::sync::Arc;

mod common_handler;
mod text_handler;

mod opt {
	#[cfg(feature = "json")]
	mod json_handler;

	#[cfg(feature = "json")]
	pub use self::json_handler::*;
}

use self::common_handler::*;
pub use self::{opt::*, text_handler::*};
use crate::{attr::Attr, context::Context, level::Level, record::Record};

/// Methods aimed at returning a copy of the handler are mutable to
/// allow implementors who desire it, to run optimizations (pre-computing,
/// caching) before forking a copy of the logger.
/// The same logic applies for `handle` who may want to defer string computation
/// until the very first time a log is issued (and then keep values cached)
pub trait Handler<L: Level, C: Context> {
	type Error;
	fn enabled(&self, ctx: C, lvl: L) -> bool;
	fn handle(&mut self, ctx: C, rec: Record<L>) -> Result<(), Self::Error>;
	fn with_attrs(&mut self, attrs: Vec<Attr>) -> Self;
	fn with_group(&mut self, name: Arc<str>) -> Self;
}
