use std::time::SystemTime;

use crate::{attr::Attr, level::Level, source::Source};

pub struct Record<T: Level> {
	time: SystemTime,
	message: String,
	level: T,
	src: Option<Source>,
	attrs: Vec<Attr>,
}
