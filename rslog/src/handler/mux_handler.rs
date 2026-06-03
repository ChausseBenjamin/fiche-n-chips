use std::any::Any;
use std::error::Error;
use std::sync::Arc;

use crate::{Attr, Handler, Level, Record};

pub struct MuxHandler(Box<[Box<dyn Handler>]>);

impl MuxHandler {
	#[must_use]
	pub fn new(handlers: Box<[Box<dyn Handler>]>) -> Self { Self(handlers) }
}

impl Handler for MuxHandler {
	fn enabled(&self, _ctx: &dyn Any, _lvl: &dyn Level) -> bool { true }

	fn handle(
		&mut self,
		ctx: &dyn Any,
		rec: Record,
	) -> Result<(), Box<dyn Error + Send + Sync>> {
		let mut first_err = None;
		for h in &mut self.0 {
			if h.enabled(ctx, &*rec.level) {
				if let Err(e) = h.handle(ctx, rec.clone()) {
					if first_err.is_none() {
						first_err = Some(e);
					}
				}
			}
		}
		match first_err {
			Some(e) => Err(e),
			None => Ok(()),
		}
	}

	fn with_attrs(&mut self, attrs: Vec<Attr>) -> Box<dyn Handler> {
		let children: Vec<Box<dyn Handler>> =
			self.0.iter_mut().map(|h| h.with_attrs(attrs.clone())).collect();
		Box::new(MuxHandler(children.into_boxed_slice()))
	}

	fn with_group(&mut self, name: Arc<str>) -> Box<dyn Handler> {
		let children: Vec<Box<dyn Handler>> =
			self.0.iter_mut().map(|h| h.with_group(name.clone())).collect();
		Box::new(MuxHandler(children.into_boxed_slice()))
	}
}

#[cfg(test)]
mod tests {
	use std::io::{self, Write};
	use std::sync::{Arc, Mutex};
	use std::time::SystemTime;

	use super::*;
	use crate::handler::TextHandler;
	use crate::{attr, Record, Source, DEBUG, ERROR, INFO, WARN};

	struct Buf(Arc<Mutex<Vec<u8>>>);

	impl Write for Buf {
		fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
			self.0.lock().unwrap().write(buf)
		}

		fn flush(&mut self) -> io::Result<()> { self.0.lock().unwrap().flush() }
	}

	fn buf() -> (Buf, Arc<Mutex<Vec<u8>>>) {
		let inner = Arc::new(Mutex::new(Vec::new()));
		(Buf(inner.clone()), inner)
	}

	fn record(msg: &str) -> Record {
		Record {
			time:  SystemTime::UNIX_EPOCH,
			msg:   Arc::from(msg),
			level: Arc::new(INFO),
			src:   Source::new("m", "f", 1),
			attrs: vec![],
		}
	}

	#[test]
	fn mux_enabled_always_true() {
		let txt1 = TextHandler::new(Vec::new()).min_level(WARN.value());
		let txt2 = TextHandler::new(Vec::new()).min_level(ERROR.value());
		let mux = MuxHandler::new(Box::new([Box::new(txt1), Box::new(txt2)]));

		assert!(mux.enabled(&(), &DEBUG));
		assert!(mux.enabled(&(), &INFO));
		assert!(mux.enabled(&(), &WARN));
		assert!(mux.enabled(&(), &ERROR));
	}

	#[test]
	fn mux_forwards_record_to_all_children() {
		let (w1, b1) = buf();
		let (w2, b2) = buf();
		let txt1 = TextHandler::new(w1).attach_time(false).min_level(DEBUG.value());
		let txt2 = TextHandler::new(w2).attach_time(false).min_level(DEBUG.value());
		let mut mux = MuxHandler::new(Box::new([Box::new(txt1), Box::new(txt2)]));

		mux.handle(&(), record("hello")).unwrap();

		assert_eq!(
			String::from_utf8(b1.lock().unwrap().clone()).unwrap(),
			"INFO hello: \n"
		);
		assert_eq!(
			String::from_utf8(b2.lock().unwrap().clone()).unwrap(),
			"INFO hello: \n"
		);
	}

	#[test]
	fn mux_with_attrs_propagates_to_children() {
		let (w1, b1) = buf();
		let (w2, b2) = buf();
		let txt1 = TextHandler::new(w1).attach_time(false).min_level(DEBUG.value());
		let txt2 = TextHandler::new(w2).attach_time(false).min_level(DEBUG.value());
		let mut mux = MuxHandler::new(Box::new([Box::new(txt1), Box::new(txt2)]));

		let mut forked = mux.with_attrs(vec![attr!("key", "val")]);
		forked.handle(&(), record("forked")).unwrap();

		assert!(String::from_utf8(b1.lock().unwrap().clone())
			.unwrap()
			.contains("key=\"val\""));
		assert!(String::from_utf8(b2.lock().unwrap().clone())
			.unwrap()
			.contains("key=\"val\""));
	}

	#[test]
	fn mux_with_group_propagates_to_children() {
		let (w1, b1) = buf();
		let (w2, b2) = buf();
		let txt1 = TextHandler::new(w1).attach_time(false).min_level(DEBUG.value());
		let txt2 = TextHandler::new(w2).attach_time(false).min_level(DEBUG.value());
		let mut mux = MuxHandler::new(Box::new([Box::new(txt1), Box::new(txt2)]));

		let mut forked = mux.with_attrs(vec![attr!("inner", "val")]);
		let mut grouped = forked.with_group("ns".into());
		grouped.handle(&(), record("grouped")).unwrap();

		assert!(String::from_utf8(b1.lock().unwrap().clone())
			.unwrap()
			.contains("ns.inner=\"val\""));
		assert!(String::from_utf8(b2.lock().unwrap().clone())
			.unwrap()
			.contains("ns.inner=\"val\""));
	}

	#[cfg(feature = "json")]
	#[test]
	fn mux_text_and_json() {
		use serde_json::Value;

		use crate::handler::JsonHandler;

		let (txt_w, txt_buf) = buf();
		let (json_w, json_buf) = buf();
		let txt =
			TextHandler::new(txt_w).attach_time(false).min_level(DEBUG.value());
		let json =
			JsonHandler::new(json_w).attach_time(false).min_level(DEBUG.value());
		let mut mux = MuxHandler::new(Box::new([Box::new(txt), Box::new(json)]));

		mux.handle(&(), record("hello")).unwrap();

		assert_eq!(
			String::from_utf8(txt_buf.lock().unwrap().clone()).unwrap(),
			"INFO hello: \n"
		);

		let json_val: Value =
			serde_json::from_slice(&json_buf.lock().unwrap()).unwrap();
		assert_eq!(json_val["msg"], "hello");
		assert_eq!(json_val["level"], "INFO");
	}
}
