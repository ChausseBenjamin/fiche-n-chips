use std::{any::Any, error::Error, sync::Arc, time::SystemTime};

use crate::{
	attr::Attr,
	context::{Background, Context as Ctx},
	source::Source as Src,
	Handler, Level as Lvl, Record, DEBUG, ERROR, INFO, WARN,
};

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

	pub fn log_context<C: Ctx + 'static>(
		&mut self,
		ctx: &C,
		level: impl Lvl + 'static,
		msg: Arc<str>,
		src_args: (Src, Vec<Attr>),
	) -> Result<(), Box<dyn Error + Send + Sync>> {
		let lvl: Arc<dyn Lvl> = Arc::new(level);
		let ctx: &dyn Any = ctx;
		if !self.0.enabled(ctx, &*lvl) {
			return Ok(());
		}
		let rec = Record {
			time: SystemTime::now(),
			msg,
			level: lvl,
			src: src_args.0,
			attrs: src_args.1,
		};

		self.0.handle(ctx, rec)
	}

  #[rustfmt::skip]
	pub fn debug_context<C: Ctx + 'static>(&mut self, ctx: &C, msg: Arc<str>, src_args: (Src, Vec<Attr>))
		  -> Result<(), Box<dyn Error + Send + Sync>> {
		self.log_context(ctx, DEBUG, msg, src_args)
	}

  #[rustfmt::skip]
	pub fn info_context<C: Ctx + 'static>(&mut self, ctx: &C, msg: Arc<str>, src_args: (Src, Vec<Attr>))
		  -> Result<(), Box<dyn Error + Send + Sync>> {
		self.log_context(ctx, INFO, msg, src_args)
	}

  #[rustfmt::skip]
	pub fn warn_context<C: Ctx + 'static>(&mut self, ctx: &C, msg: Arc<str>, src_args: (Src, Vec<Attr>))
		  -> Result<(), Box<dyn Error + Send + Sync>> {
		self.log_context(ctx, WARN, msg, src_args)
	}

  #[rustfmt::skip]
	pub fn error_context<C: Ctx + 'static>(&mut self, ctx: &C, msg: Arc<str>, src_args: (Src, Vec<Attr>))
		  -> Result<(), Box<dyn Error + Send + Sync>> {
		self.log_context(ctx, ERROR, msg, src_args)
	}

  #[rustfmt::skip]
	pub fn log(&mut self, level: impl Lvl + 'static, msg: Arc<str>, src_args: (Src, Vec<Attr>))
		  -> Result<(), Box<dyn Error + Send + Sync>> {
		self.log_context(&Background, level, msg, src_args)
	}

  #[rustfmt::skip]
	pub fn debug(&mut self, msg: Arc<str>, src_args: (Src, Vec<Attr>))
		  -> Result<(), Box<dyn Error + Send + Sync>> {
		self.log_context(&Background, DEBUG, msg, src_args)
	}

  #[rustfmt::skip]
	pub fn info(&mut self, msg: Arc<str>, src_args: (Src, Vec<Attr>))
		  -> Result<(), Box<dyn Error + Send + Sync>> {
		self.log_context(&Background, INFO, msg, src_args)
	}

  #[rustfmt::skip]
	pub fn warn(&mut self, msg: Arc<str>, src_args: (Src, Vec<Attr>))
		  -> Result<(), Box<dyn Error + Send + Sync>> {
		self.log_context(&Background, WARN, msg, src_args)
	}

  #[rustfmt::skip]
	pub fn error(&mut self, msg: Arc<str>, src_args: (Src, Vec<Attr>))
		  -> Result<(), Box<dyn Error + Send + Sync>> {
		self.log_context(&Background, ERROR, msg, src_args)
	}
}
