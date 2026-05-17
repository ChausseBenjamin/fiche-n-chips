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

	pub fn log<C: Context>(
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
}

#[macro_export]
macro_rules! log_context {
    ($logger:expr, $ctx:expr, $level:expr, $msg:expr $(, $($attrs:tt)*)?) => {
        $logger.log(
            $ctx,
            $level,
            $msg.into(),
            $crate::source!(),
            vec![$crate::attrs!($($($attrs)*)?)]
        )
    };
}
