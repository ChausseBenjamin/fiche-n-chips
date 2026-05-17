use std::sync::Arc;

mod common_handler;
mod discard_handler;
mod text_handler;

use self::common_handler::*;
#[cfg(feature = "json")]
pub use self::opt::*;
pub use self::{discard_handler::*, text_handler::*};
use crate::{attr::Attr, context::Context, level::Level, record::Record};

mod opt {
	#[cfg(feature = "json")]
	mod json_handler;

	#[cfg(feature = "json")]
	pub use self::json_handler::*;
}

/// Methods aimed at returning a copy of the handler are mutable to
/// allow implementors who desire it, to run optimizations (pre-computing,
/// caching) before forking a copy of the logger.
/// The same logic applies for `handle` who may want to defer string computation
/// until the very first time a log is issued (and then keep values cached)
pub trait Handler {
	type Error;
	fn enabled<C: Context>(&self, ctx: &C, lvl: &dyn Level) -> bool;
	fn handle<C: Context>(
		&mut self,
		ctx: &C,
		rec: Record,
	) -> Result<(), Self::Error>;
	fn with_attrs(&mut self, attrs: Vec<Attr>) -> Self;
	fn with_group(&mut self, name: Arc<str>) -> Self;
}
