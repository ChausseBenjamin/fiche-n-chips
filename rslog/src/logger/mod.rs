use std::any::Any;
use std::sync::Arc;
use std::time::SystemTime;

use crate::attr::Attr;
use crate::context::{Background, Context as Ctx};
use crate::source::Source as Src;
use crate::{Handler, Level as Lvl, Record, DEBUG, ERROR, INFO, WARN};

pub struct Logger(Box<dyn Handler>);

impl<H: Handler + 'static> From<H> for Logger {
	fn from(handler: H) -> Self {
		Self(Box::new(handler))
	}
}

impl Logger {
	pub fn new(handler: Box<dyn Handler>) -> Self {
		Self(handler)
	}

	pub fn handler(&mut self) -> Box<dyn Handler> {
		// HACK: can't `Copy` a Box<dyn Handler> but calling with_attrs with an
		// empty vector essentially forces the underlying handler to clone itself.
		self.0.with_attrs(vec![])
	}

	pub fn enabled(&self, ctx: &dyn Any, level: &dyn Lvl) -> bool {
		self.0.enabled(ctx, level)
	}

	pub fn with(&mut self, attrs: Vec<Attr>) -> Self {
		Self(self.0.with_attrs(attrs))
	}

	pub fn log_context<C: Ctx + 'static>(
		&mut self,
		ctx: &C,
		level: impl Lvl + 'static,
		msg: &str,
		src_args: (Src, Vec<Attr>),
	) {
		let lvl: Arc<dyn Lvl> = Arc::new(level);
		let ctx: &dyn Any = ctx;
		if !self.0.enabled(ctx, &*lvl) {
			return;
		}
		let rec = Record {
			time:  SystemTime::now(),
			msg:   Arc::from(msg),
			level: lvl,
			src:   src_args.0,
			attrs: src_args.1,
		};

		let _ = self.0.handle(ctx, rec);
	}

	pub fn debug_context<C: Ctx + 'static>(&mut self, ctx: &C, msg: &str, src_args: (Src, Vec<Attr>)) {
		self.log_context(ctx, DEBUG, msg, src_args);
	}

	pub fn info_context<C: Ctx + 'static>(&mut self, ctx: &C, msg: &str, src_args: (Src, Vec<Attr>)) {
		self.log_context(ctx, INFO, msg, src_args);
	}

	pub fn warn_context<C: Ctx + 'static>(&mut self, ctx: &C, msg: &str, src_args: (Src, Vec<Attr>)) {
		self.log_context(ctx, WARN, msg, src_args);
	}

	pub fn error_context<C: Ctx + 'static>(&mut self, ctx: &C, msg: &str, src_args: (Src, Vec<Attr>)) {
		self.log_context(ctx, ERROR, msg, src_args);
	}

	pub fn log(&mut self, level: impl Lvl + 'static, msg: &str, src_args: (Src, Vec<Attr>)) {
		self.log_context(&Background, level, msg, src_args);
	}

	pub fn debug(&mut self, msg: &str, src_args: (Src, Vec<Attr>)) {
		self.log_context(&Background, DEBUG, msg, src_args);
	}

	pub fn info(&mut self, msg: &str, src_args: (Src, Vec<Attr>)) {
		self.log_context(&Background, INFO, msg, src_args);
	}

	pub fn warn(&mut self, msg: &str, src_args: (Src, Vec<Attr>)) {
		self.log_context(&Background, WARN, msg, src_args);
	}

	pub fn error(&mut self, msg: &str, src_args: (Src, Vec<Attr>)) {
		self.log_context(&Background, ERROR, msg, src_args);
	}
}
