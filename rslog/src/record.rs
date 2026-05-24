use std::sync::Arc;
use std::time::SystemTime;

use crate::attr::Attr;
use crate::level::Level;
use crate::source::Source;

#[derive(Clone)]
pub struct Record {
	pub time:  SystemTime,
	// Box<str> would be sufficient but would prohibit handler multiplexing
	pub msg:   Arc<str>,
	pub level: Arc<dyn Level>,
	pub src:   Source,
	pub attrs: Vec<Attr>,
}
