use std::sync::Arc;

use crate::Attr;

pub(super) struct CommonHandler {
	// can be useful in for sorting if multiple apps log in the same stream
	app_name: Option<Arc<str>>,
	min_level: i8,
	attach_time: bool,
	attach_source: bool,
	writer: W,

	// This is a low-effort implementation leaving lots of performance on the
	// table. Pre-computing attrs belonging to a handler as ready-made string
	// shards could greatly diminish the compute cost of every log statement
	// that uses it.
	// I want something that works first, optimize performance second as this
	// is just an implementation detail which isn't user-facing/accessible.
	attrs: Arc<[Attr]>,
}
