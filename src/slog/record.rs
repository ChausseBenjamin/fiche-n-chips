use std::time::SystemTime;

use crate::{attr::Attr, level::Level, source::Source};

pub struct Record<L: Level> {
	pub time: SystemTime,
	pub msg: String,
	pub level: L,
	pub src: Option<Source>,
	pub attrs: Vec<Attr>,
}
