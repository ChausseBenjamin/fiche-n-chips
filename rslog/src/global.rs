use std::sync::{Mutex, MutexGuard, OnceLock};

use crate::{Logger, TextHandler};

fn global() -> &'static Mutex<Logger> {
	static LOGGER: OnceLock<Mutex<Logger>> = OnceLock::new();
	LOGGER.get_or_init(|| {
		Mutex::new(Logger::new(Box::new(TextHandler::new(std::io::stdout()))))
	})
}

pub fn default() -> MutexGuard<'static, Logger> {
	global().lock().expect("global logger lock poisoned")
}

pub fn set_default(logger: Logger) {
	*global().lock().expect("global logger lock poisoned") = logger;
}
