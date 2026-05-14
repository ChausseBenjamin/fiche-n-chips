use std::time::SystemTime;

use crate::{
	Attr, Context, Handler, Level, Record, Source, DEBUG, ERROR, INFO, WARN,
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

	pub fn log<L: Level, C: Context>(
		&mut self,
		ctx: &C,
		level: L,
		msg: String,
		src: Source,
		args: Vec<Attr>,
	) -> Result<(), <H as Handler>::Error>
	where
		H: Handler,
	{
		if !self.0.enabled(ctx, &level) {
			return Ok(());
		}
		let rec = Record {
			time: SystemTime::now(),
			msg,
			level,
			src,
			attrs: args,
		};

		self.0.handle(ctx, rec)
	}
}
