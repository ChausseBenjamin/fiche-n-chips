use std::time::SystemTime;

use crate::attr::Attr;

pub struct Record {
	time: SystemTime,
	message: String,
	// TODO: level
	// TODO: source/caller
	attrs: Vec<Attr>,
}
