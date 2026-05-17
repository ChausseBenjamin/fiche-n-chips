use std::{sync::Arc, time::SystemTime};

use crate::{
	Attr, Background as Bg, Context as Ctx, Handler, Level as Lvl, Record,
	Source as Src, DEBUG, ERROR, INFO, WARN,
};

#[macro_export]
macro_rules! attrs {
    ($($rest:tt)*) => {
        $crate::source!(), $crate::__log_kvpairs!($($rest)*).to_vec()
    };
}

pub struct Logger<H>(H);

impl<H> From<H> for Logger<H> {
	fn from(handler: H) -> Self {
		Self(handler)
	}
}

impl<H> Logger<H> {
	pub fn new(handler: H) -> Self {
		Self(handler)
	}

	pub fn log_context<C: Ctx>(
		&mut self,
		ctx: &C,
		level: impl Lvl + 'static,
		msg: Arc<str>,
		src: Src,
		args: Vec<Attr>,
	) -> Result<(), <H as Handler>::Error>
	where
		H: Handler,
	{
		let lvl: Arc<dyn Lvl> = Arc::new(level);
		if !self.0.enabled(ctx, &*lvl) {
			Ok(())
		} else {
			let rec = Record {
				time: SystemTime::now(),
				msg,
				level: lvl,
				src,
				attrs: args,
			};

			self.0.handle(ctx, rec)
		}
	}

  #[rustfmt::skip]
	pub fn debug_context<C: Ctx>(&mut self, ctx: &C, msg: Arc<str>, src: Src, args: Vec<Attr>)
      -> Result<(), <H as Handler>::Error>
	where H: Handler {
		self.log_context(ctx, DEBUG, msg, src, args)
	}

  #[rustfmt::skip]
	pub fn info_context<C: Ctx>(&mut self, ctx: &C, msg: Arc<str>, src: Src, args: Vec<Attr>)
      -> Result<(), <H as Handler>::Error>
	where H: Handler {
		self.log_context(ctx, INFO, msg, src, args)
	}

  #[rustfmt::skip]
	pub fn warn_context<C: Ctx>(&mut self, ctx: &C, msg: Arc<str>, src: Src, args: Vec<Attr>)
      -> Result<(), <H as Handler>::Error>
	where H: Handler {
		self.log_context(ctx, WARN, msg, src, args)
	}

  #[rustfmt::skip]
	pub fn error_context<C: Ctx>(&mut self, ctx: &C, msg: Arc<str>, src: Src, args: Vec<Attr>)
      -> Result<(), <H as Handler>::Error>
	where H: Handler {
		self.log_context(ctx, ERROR, msg, src, args)
	}

  #[rustfmt::skip]
	pub fn log(&mut self, level: impl Lvl + 'static, msg: Arc<str>, src: Src, args: Vec<Attr>)
      -> Result<(), <H as Handler>::Error>
	where H: Handler, {
		self.log_context(&Bg {}, level, msg, src, args)
	}

  #[rustfmt::skip]
	pub fn debug(&mut self, msg: Arc<str>, src: Src, args: Vec<Attr>)
      -> Result<(), <H as Handler>::Error>
	where H: Handler {
		self.log_context(&Bg {}, DEBUG, msg, src, args)
	}

  #[rustfmt::skip]
	pub fn info(&mut self, msg: Arc<str>, src: Src, args: Vec<Attr>)
      -> Result<(), <H as Handler>::Error>
	where H: Handler {
		self.log_context(&Bg {}, INFO, msg, src, args)
	}

  #[rustfmt::skip]
	pub fn warn(&mut self, msg: Arc<str>, src: Src, args: Vec<Attr>)
      -> Result<(), <H as Handler>::Error>
	where H: Handler {
		self.log_context(&Bg {}, WARN, msg, src, args)
	}

  #[rustfmt::skip]
	pub fn error(&mut self, msg: Arc<str>, src: Src, args: Vec<Attr>)
      -> Result<(), <H as Handler>::Error>
	where H: Handler {
		self.log_context(&Bg {}, ERROR, msg, src, args)
	}
}
