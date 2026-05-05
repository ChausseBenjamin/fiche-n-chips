use super::{Attr, Context, Level, Record};

pub trait Handler {
	fn enabled(&self, ctx: &dyn Context, lvl: &dyn Level) -> bool;
	fn handle(&self, ctx: &dyn Context, rec: Record) -> Result<(), Box<dyn std::error::Error>>;
	fn is_tracking(&self) -> bool;
	fn with_attrs(&self, attrs: Vec<Attr>) -> Box<dyn Handler>;
	fn with_group(&self, name: String) -> Box<dyn Handler>;
}
