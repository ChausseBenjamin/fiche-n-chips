mod text_handler;

use std::sync::Arc;

pub use self::text_handler::*;
use crate::{attr::Attr, context::Context, level::Level, record::Record};

pub trait Handler<L: Level, C: Context> {
	type Error;
	fn enabled(&self, ctx: C, lvl: L) -> bool;
	fn handle(&self, ctx: C, rec: Record<L>) -> Result<(), Self::Error>;
	fn with_attrs(&self, attrs: Arc<[Attr]>) -> Self;
	fn with_group(&self, name: Arc<str>) -> Self;
}
