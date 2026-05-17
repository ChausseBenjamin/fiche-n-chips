use std::{sync::Arc, time::SystemTime};

use crate::{attr::Attr, level::Level, source::Source};

#[derive(Clone)]
pub struct Record {
	pub time: SystemTime,
	// Box<str> would be sufficient but would prohibit handler multiplexing
	pub msg: Arc<str>,
	pub level: Arc<dyn Level>,
	pub src: Source,
	pub attrs: Vec<Attr>,
}
