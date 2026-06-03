use std::any::Any;
use std::error::Error;
use std::sync::Arc;

mod common_handler;
mod discard_handler;
mod mux_handler;
mod text_handler;

use self::common_handler::CommonHandler;
pub use self::discard_handler::*;
pub use self::mux_handler::*;
pub use self::text_handler::*;
use crate::attr::Attr;
use crate::level::Level;
use crate::record::Record;

#[cfg(feature = "json")]
#[allow(unused_imports)]
mod opt {
	pub use self::json_handler::*;
	mod json_handler;
}

#[cfg(feature = "json")]
#[allow(unused_imports)]
pub use self::opt::*;

/// Methods aimed at returning a copy of the handler are mutable to
/// allow implementors to run optimizations (pre-computing/caching)
/// before forking a copy of the logger.
/// The same logic applies for `handle` who may want to defer string computation
/// until the very first time a log is issued (and then keep values cached)
///
/// # Errors
///
/// Returns an error if the underlying output sink fails to write the log
/// record.
pub trait Handler: Send {
	fn enabled(&self, ctx: &dyn Any, lvl: &dyn Level) -> bool;
	#[allow(clippy::missing_errors_doc)]
	fn handle(
		&mut self,
		ctx: &dyn Any,
		rec: Record,
	) -> Result<(), Box<dyn Error + Send + Sync>>;
	fn with_attrs(&mut self, attrs: Vec<Attr>) -> Box<dyn Handler>;
	fn with_group(&mut self, name: Arc<str>) -> Box<dyn Handler>;
}
