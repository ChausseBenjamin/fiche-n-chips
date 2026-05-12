use std::time::SystemTime;

use crate::{attr::Attr, level::Level, source::Source};

pub struct Record<L: Level> {
	time: SystemTime,
	message: String,
	level: L,
	src: Option<Source>,
	attrs: Vec<Attr>,
}
