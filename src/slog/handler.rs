use std::error::Error;

use crate::{
	context::Context,
	level::{Level, Lvl},
	record::Record,
};

pub trait Handler {
	type Error: Error;
	fn enabled(&self, ctx: Context, lvl: Level) -> bool;
	fn handle(&self, ctx: Context, rec: Record) -> Result<(), Error>;
	fn with_attrs(&self, attrs: Arc<[Attr]>) -> Handler;
	fn with_group(&self, name: Arc<str>) -> Handler;
}
