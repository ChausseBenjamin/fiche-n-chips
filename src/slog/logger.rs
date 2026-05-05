use std::{
	panic::Location,
	sync::{OnceLock, RwLock, RwLockReadGuard},
	time::SystemTime,
};

use super::{
	attrs, context::Background, Attr, Caller, Context, Handler, Level, Record,
};
use crate::level::{DEBUG, ERROR, INFO, WARN};

static DEFAULT_LOGGER: OnceLock<RwLock<Logger>> = OnceLock::new();

fn init() -> RwLock<Logger> {
	RwLock::new(Logger {
		handler: Box::new(DummyHandler),
	})
}

struct DummyHandler;

impl Handler for DummyHandler {
	fn enabled(&self, _ctx: &dyn Context, _lvl: &dyn Level) -> bool {
		true
	}
	fn handle(
		&self,
		_ctx: &dyn Context,
		_rec: Record,
	) -> Result<(), Box<dyn std::error::Error>> {
		Ok(())
	}
	fn is_tracking(&self) -> bool {
		true
	}
	fn with_attrs(&self, _attrs: Vec<Attr>) -> Box<dyn Handler> {
		Box::new(DummyHandler)
	}
	fn with_group(&self, _name: String) -> Box<dyn Handler> {
		Box::new(DummyHandler)
	}
}

pub fn default() -> std::sync::RwLockReadGuard<'static, Logger> {
	match DEFAULT_LOGGER.get_or_init(init).read() {
		Ok(guard) => guard,
		// I'd rather deal with a poisoned logger than break an app because of it
		Err(poisoned) => poisoned.into_inner(),
	}
}

pub struct Logger {
	handler: Box<dyn Handler + Send + Sync>,
}

impl Logger {
	#[track_caller]
	fn log_context_internal(
		&self,
		ctx: &dyn Context,
		lvl: impl Level + 'static,
		msg: String,
		loc: Option<Caller>,
		args: Vec<Attr>,
	) -> Record {
		Record::new(
			SystemTime::now(),
			lvl,
			msg,
			if self.handler.is_tracking() {
				loc
			} else {
				None
			},
			args,
		)
	}

	#[track_caller]
	pub fn log_context(
		&self,
		ctx: &dyn Context,
		lvl: impl Level + 'static,
		msg: impl Into<String>,
		args: Vec<Attr>,
	) {
		let location = Location::caller();
		let caller = Caller::new(location.file(), location.file(), location.line());
		let record =
			self.log_context_internal(ctx, lvl, msg.into(), Some(caller), args);
		// Assume handler handles it
		let _ = self.handler.handle(ctx, record);
	}

	#[track_caller]
	pub fn log(
		&self,
		lvl: impl Level + 'static,
		msg: impl Into<String>,
		args: Vec<Attr>,
	) {
		self.log_context(&Background::new(), lvl, msg, args);
	}

	#[track_caller]
	pub fn debug_context(
		&self,
		ctx: &dyn Context,
		msg: impl Into<String>,
		args: Vec<Attr>,
	) {
		self.log_context(ctx, DEBUG, msg, args);
	}

	#[track_caller]
	pub fn debug(&self, msg: impl Into<String>, args: Vec<Attr>) {
		self.log_context(&Background::new(), DEBUG, msg, args);
	}

	#[track_caller]
	pub fn info_context(
		&self,
		ctx: &dyn Context,
		msg: impl Into<String>,
		args: Vec<Attr>,
	) {
		self.log_context(ctx, INFO, msg, args);
	}

	#[track_caller]
	pub fn info(&self, msg: impl Into<String>, args: Vec<Attr>) {
		self.log_context(&Background::new(), INFO, msg, args);
	}

	#[track_caller]
	pub fn warn_context(
		&self,
		ctx: &dyn Context,
		msg: impl Into<String>,
		args: Vec<Attr>,
	) {
		self.log_context(ctx, WARN, msg, args);
	}

	#[track_caller]
	pub fn warn(&self, msg: impl Into<String>, args: Vec<Attr>) {
		self.log_context(&Background::new(), WARN, msg, args);
	}

	#[track_caller]
	pub fn error_context(
		&self,
		ctx: &dyn Context,
		msg: impl Into<String>,
		args: Vec<Attr>,
	) {
		self.log_context(ctx, ERROR, msg, args);
	}

	#[track_caller]
	pub fn error(&self, msg: impl Into<String>, args: Vec<Attr>) {
		self.log_context(&Background::new(), ERROR, msg, args);
	}
}

#[track_caller]
pub fn log_context(ctx: &dyn Context, lvl: impl Level + 'static, msg: impl Into<String>, args: Vec<Attr>) {
	default().log_context(ctx, lvl, msg, args);
}

#[track_caller]
pub fn log(lvl: impl Level + 'static, msg: impl Into<String>, args: Vec<Attr>) {
	default().log(lvl, msg, args);
}

#[track_caller]
pub fn debug_context(ctx: &dyn Context, msg: impl Into<String>, args: Vec<Attr>) {
	default().debug_context(ctx, msg, args);
}

#[track_caller]
pub fn debug(msg: impl Into<String>, args: Vec<Attr>) {
	default().debug(msg, args);
}

#[track_caller]
pub fn info_context(ctx: &dyn Context, msg: impl Into<String>, args: Vec<Attr>) {
	default().info_context(ctx, msg, args);
}

#[track_caller]
pub fn info(msg: impl Into<String>, args: Vec<Attr>) {
	default().info(msg, args);
}

#[track_caller]
pub fn warn_context(ctx: &dyn Context, msg: impl Into<String>, args: Vec<Attr>) {
	default().warn_context(ctx, msg, args);
}

#[track_caller]
pub fn warn(msg: impl Into<String>, args: Vec<Attr>) {
	default().warn(msg, args);
}

#[track_caller]
pub fn error_context(ctx: &dyn Context, msg: impl Into<String>, args: Vec<Attr>) {
	default().error_context(ctx, msg, args);
}

#[track_caller]
pub fn error(msg: impl Into<String>, args: Vec<Attr>) {
	default().error(msg, args);
}
