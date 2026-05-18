use std::{
	any::Any,
	error::Error,
	ops::{Deref, DerefMut},
	sync::{Arc, Mutex, MutexGuard, OnceLock},
};

use crate::{
	Attr, Context as Ctx, Level as Lvl, Logger, Source as Src, TextHandler,
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

pub fn log_context<C: Ctx + 'static>(
	ctx: &C,
	level: impl Lvl + 'static,
	msg: Arc<str>,
	src_args: (Src, Vec<Attr>),
) -> Result<(), Box<dyn Error + Send + Sync>> {
	default().log_context(ctx, level, msg, src_args)
}

#[rustfmt::skip]
pub fn debug_context<C: Ctx + 'static>(ctx: &C, msg: Arc<str>, src_args: (Src, Vec<Attr>))
	-> Result<(), Box<dyn Error + Send + Sync>> {
	default().debug_context(ctx, msg, src_args)
}

#[rustfmt::skip]
pub fn info_context<C: Ctx + 'static>(ctx: &C, msg: Arc<str>, src_args: (Src, Vec<Attr>))
	-> Result<(), Box<dyn Error + Send + Sync>> {
	default().info_context(ctx, msg, src_args)
}

#[rustfmt::skip]
pub fn warn_context<C: Ctx + 'static>(ctx: &C, msg: Arc<str>, src_args: (Src, Vec<Attr>))
	-> Result<(), Box<dyn Error + Send + Sync>> {
	default().warn_context(ctx, msg, src_args)
}

#[rustfmt::skip]
pub fn error_context<C: Ctx + 'static>(ctx: &C, msg: Arc<str>, src_args: (Src, Vec<Attr>))
	-> Result<(), Box<dyn Error + Send + Sync>> {
	default().error_context(ctx, msg, src_args)
}

#[rustfmt::skip]
pub fn log(level: impl Lvl + 'static, msg: Arc<str>, src_args: (Src, Vec<Attr>))
	-> Result<(), Box<dyn Error + Send + Sync>> {
	default().log(level, msg, src_args)
}

#[rustfmt::skip]
pub fn debug(msg: Arc<str>, src_args: (Src, Vec<Attr>))
	-> Result<(), Box<dyn Error + Send + Sync>> {
	default().debug(msg, src_args)
}

#[rustfmt::skip]
pub fn info(msg: Arc<str>, src_args: (Src, Vec<Attr>))
	-> Result<(), Box<dyn Error + Send + Sync>> {
	default().info(msg, src_args)
}

#[rustfmt::skip]
pub fn warn(msg: Arc<str>, src_args: (Src, Vec<Attr>))
	-> Result<(), Box<dyn Error + Send + Sync>> {
	default().warn(msg, src_args)
}

#[rustfmt::skip]
pub fn error(msg: Arc<str>, src_args: (Src, Vec<Attr>))
	-> Result<(), Box<dyn Error + Send + Sync>> {
	default().error(msg, src_args)
}
