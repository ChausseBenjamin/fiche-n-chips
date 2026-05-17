use std::time::SystemTime;

use crate::{attr::Attr, level::Level, source::Source};

pub struct Record {
	pub time: SystemTime,
	pub msg: String,
	pub level: Box<dyn Level>,
	pub src: Source,
	pub attrs: Vec<Attr>,
}
