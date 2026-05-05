use std::time::SystemTime;

use super::{attrs, Attr, Level};
use crate::level;

pub struct Record {
	pub time: SystemTime,
	pub level: Box<dyn Level>,
	pub msg: String,
	pub caller: Option<Caller>,
	// NOTE: go optimizes this with front/back where front is fixed N size and
	// pre-allocated. This ensures allocations are only needed for exceptional
	// cases where total attrs exceed N.
	pub attrs: Vec<Attr>,
}

pub struct Caller {
	module: &'static str,
	file: &'static str,
	line: u32,
}

impl Caller {
	pub fn new(module: &'static str, file: &'static str, line: u32) -> Self {
		Self { module, file, line }
	}
}

impl Record {
	pub fn new(
		time: SystemTime,
		level: impl Level + 'static,
		msg: String,
		caller: Option<Caller>,
		attrs: Vec<Attr>,
	) -> Self {
		Self {
			time,
			level: Box::new(level),
			msg,
			caller,
			attrs: attrs,
		}
	}
}
