use std::io::Write;
use std::sync::{Arc, Mutex};

use crate::{Attr, AttrGroup, Level, INFO};

// pub fn ReplaceAttr(groups: &[&str],attr: Attr) -> Attr;

pub(super) struct CommonHandler<W: Write> {
	// can be useful in for sorting if multiple apps log in the same stream
	pub app_name:      Option<Arc<str>>,
	pub min_level:     i8,
	pub attach_time:   bool,
	pub attach_source: bool,
	// XXX: might need a mutex if writers shit the bed with concurent calls
	pub writer:        Arc<Mutex<W>>,

	// This is a low-effort implementation leaving lots of performance on the
	// table. Pre-computing attrs belonging to a handler as ready-made string
	// shards could greatly diminish the compute cost of every log statement
	// that uses it.
	// I want something that works. Optimizing performance comes later.
	pub attrs: Vec<Attr>,
	// There are two approaches that *may* be worth exploring for future
	// optimizations:
	//
	// OPTION 1: Whenever a handler runs for the first time (`.handle(...)`),
	// then internal attr tree gets flattened into a slice of attr_strings (each
	// string holds both key+value, which implies all groups are also
	// flattened). Subsequent calls to the same handler use the pre-computed
	// strings. Child loggers already have part of the string data computed.
	//
	// OPTION 2: Whenever a logger is forked (with_attrs or with_group), the
	// parent pre-computes the data he owns (if not already done) for both his
	// child and himself (this is why I let the trait set `&mut self` for those
	// even if no mutation *should* occur).
	//
	// Both implementations would probably require TextHandler and JsonHandler to
	// provide serialization functions for standalone attributes and
	// groups to leave this common core implementation agnostic.
	//
	// I'm not sure which approach is best. Option 1 condemns the first log to be
	// slow, and that might occur in hot-paths. Option 2 condemns function to
	// run potentially slow pre-computations even if no log ever gets printed
	// using that specific logger.
}

impl<W: Write> Clone for CommonHandler<W> {
	fn clone(&self) -> Self {
		Self {
			app_name:      self.app_name.clone(),
			min_level:     self.min_level,
			attach_time:   self.attach_time,
			attach_source: self.attach_source,
			writer:        self.writer.clone(),
			attrs:         self.attrs.clone(),
		}
	}
}

impl<W: Write> CommonHandler<W> {
	pub fn new(writer: W) -> Self {
		Self {
			app_name:      None,
			min_level:     INFO.value(),
			attach_time:   true,
			attach_source: false,
			attrs:         Default::default(),
			writer:        Arc::new(Mutex::new(writer)),
		}
	}

	pub fn with_attrs(&self, attrs: Vec<Attr>) -> Self {
		let mut other = self.clone();
		other.attrs.extend(attrs);
		other
	}

	pub fn with_group(&self, name: Arc<str>) -> Self {
		let group = Attr::Grp(AttrGroup {
			key:   name,
			attrs: self.attrs.clone().into(),
		});
		Self {
			app_name:      self.app_name.clone(),
			min_level:     self.min_level,
			attach_time:   self.attach_time,
			attach_source: self.attach_source,
			writer:        self.writer.clone(),
			attrs:         vec![group],
		}
	}
}
