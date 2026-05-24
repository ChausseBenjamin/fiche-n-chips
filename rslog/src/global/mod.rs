use std::any::Any;
use std::ops::{Deref, DerefMut};
use std::sync::{Mutex, MutexGuard, OnceLock};

use crate::{
	Attr, Context as Ctx, Handler, Level as Lvl, Logger, Source as Src, TextHandler,
};

pub struct DefaultLogger<L>(pub(crate) L);

impl<L: Deref<Target = Logger>> Deref for DefaultLogger<L> {
	type Target = Logger;

	fn deref(&self) -> &Logger {
		self.0.deref()
	}
}

impl<L: DerefMut + Deref<Target = Logger>> DerefMut for DefaultLogger<L> {
	fn deref_mut(&mut self) -> &mut Logger {
		self.0.deref_mut()
	}
}

pub fn default() -> DefaultLogger<MutexGuard<'static, Logger>> {
	DefaultLogger(global().lock().expect("global logger lock poisoned"))
}

pub fn set_default(logger: Logger) {
	*global().lock().expect("global logger lock poisoned") = logger;
}

fn global() -> &'static Mutex<Logger> {
	static LOGGER: OnceLock<Mutex<Logger>> = OnceLock::new();
	LOGGER.get_or_init(|| {
		Mutex::new(Logger::new(Box::new(TextHandler::new(std::io::stdout()))))
	})
}

pub fn handler() -> Box<dyn Handler> {
	default().handler()
}

pub fn enabled(ctx: &dyn Any, level: &dyn Lvl) -> bool {
	default().enabled(ctx, level)
}

pub fn with(attrs: Vec<Attr>) -> Logger {
	default().with(attrs)
}

pub fn log_context<C: Ctx + 'static>(
	ctx: &C,
	level: impl Lvl + 'static,
	msg: &str,
	src_args: (Src, Vec<Attr>),
) {
	default().log_context(ctx, level, msg, src_args);
}

pub fn debug_context<C: Ctx + 'static>(ctx: &C, msg: &str, src_args: (Src, Vec<Attr>)) {
	default().debug_context(ctx, msg, src_args);
}

pub fn info_context<C: Ctx + 'static>(ctx: &C, msg: &str, src_args: (Src, Vec<Attr>)) {
	default().info_context(ctx, msg, src_args);
}

pub fn warn_context<C: Ctx + 'static>(ctx: &C, msg: &str, src_args: (Src, Vec<Attr>)) {
	default().warn_context(ctx, msg, src_args);
}

pub fn error_context<C: Ctx + 'static>(ctx: &C, msg: &str, src_args: (Src, Vec<Attr>)) {
	default().error_context(ctx, msg, src_args);
}

pub fn log(level: impl Lvl + 'static, msg: &str, src_args: (Src, Vec<Attr>)) {
	default().log(level, msg, src_args);
}

pub fn debug(msg: &str, src_args: (Src, Vec<Attr>)) {
	default().debug(msg, src_args);
}

pub fn info(msg: &str, src_args: (Src, Vec<Attr>)) {
	default().info(msg, src_args);
}

pub fn warn(msg: &str, src_args: (Src, Vec<Attr>)) {
	default().warn(msg, src_args);
}

pub fn error(msg: &str, src_args: (Src, Vec<Attr>)) {
	default().error(msg, src_args);
}
