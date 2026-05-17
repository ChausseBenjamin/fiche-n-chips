use std::time::SystemTime;

use crate::{
	Attr, Background, Context, Handler, Level, Record, Source, DEBUG, ERROR,
	INFO, WARN,
};

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

	pub fn log_context<C: Context>(
		&mut self,
		ctx: &C,
		level: impl Level + 'static,
		msg: String,
		src: Source,
		args: Vec<Attr>,
	) -> Result<(), <H as Handler>::Error>
	where
		H: Handler,
	{
		let lvl = Box::from(level);
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

	pub fn debug_context<C: Context>(
		&mut self,
		ctx: &C,
		level: impl Level + 'static,
		msg: String,
		src: Source,
		args: Vec<Attr>,
	) -> Result<(), <H as Handler>::Error>
	where
		H: Handler,
	{
		self.log_context(&Background {}, DEBUG, msg, src, args)
	}

	pub fn info_context<C: Context>(
		&mut self,
		ctx: &C,
		level: impl Level + 'static,
		msg: String,
		src: Source,
		args: Vec<Attr>,
	) -> Result<(), <H as Handler>::Error>
	where
		H: Handler,
	{
		self.log_context(&Background {}, INFO, msg, src, args)
	}

	pub fn warn_context<C: Context>(
		&mut self,
		ctx: &C,
		level: impl Level + 'static,
		msg: String,
		src: Source,
		args: Vec<Attr>,
	) -> Result<(), <H as Handler>::Error>
	where
		H: Handler,
	{
		self.log_context(&Background {}, WARN, msg, src, args)
	}

	pub fn error_context<C: Context>(
		&mut self,
		ctx: &C,
		level: impl Level + 'static,
		msg: String,
		src: Source,
		args: Vec<Attr>,
	) -> Result<(), <H as Handler>::Error>
	where
		H: Handler,
	{
		self.log_context(&Background {}, ERROR, msg, src, args)
	}

	pub fn log(
		&mut self,
		level: impl Level + 'static,
		msg: String,
		src: Source,
		args: Vec<Attr>,
	) -> Result<(), <H as Handler>::Error>
	where
		H: Handler,
	{
		self.log_context(&Background {}, level, msg, src, args)
	}

	pub fn debug(
		&mut self,
		level: impl Level + 'static,
		msg: String,
		src: Source,
		args: Vec<Attr>,
	) -> Result<(), <H as Handler>::Error>
	where
		H: Handler,
	{
		self.log_context(&Background {}, DEBUG, msg, src, args)
	}

	pub fn info(
		&mut self,
		level: impl Level + 'static,
		msg: String,
		src: Source,
		args: Vec<Attr>,
	) -> Result<(), <H as Handler>::Error>
	where
		H: Handler,
	{
		self.log_context(&Background {}, INFO, msg, src, args)
	}

	pub fn warn(
		&mut self,
		level: impl Level + 'static,
		msg: String,
		src: Source,
		args: Vec<Attr>,
	) -> Result<(), <H as Handler>::Error>
	where
		H: Handler,
	{
		self.log_context(&Background {}, WARN, msg, src, args)
	}

	pub fn error(
		&mut self,
		level: impl Level + 'static,
		msg: String,
		src: Source,
		args: Vec<Attr>,
	) -> Result<(), <H as Handler>::Error>
	where
		H: Handler,
	{
		self.log_context(&Background {}, ERROR, msg, src, args)
	}
}

#[macro_export]
macro_rules! attrs {
    ($($rest:tt)*) => {
        $crate::source!(), $crate::__log_kvpairs!($($rest)*).to_vec()
    };
}
