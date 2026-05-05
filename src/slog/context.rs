use std::fmt::Display;

use super::log_valuer::LogValuer;

// TODO: This is probarly bad code, I just needed something temporary so my
// methods in the Handler trait could have a Context attribute...
// TODO: This abstraction must work accross both actix_web and axum to emulate
// a Go context

pub enum LogValue {
	Lazy(Box<dyn LogValuer>),
	Display(Box<dyn Display + Send + Sync>),
}

pub trait Context {
	fn get(&self, key: &str) -> Option<LogValue>;
}

pub struct Background;

impl Background {
	pub fn new() -> Self {
		Self
	}
}

impl Context for Background {
	fn get(&self, _key: &str) -> Option<LogValue> {
		None
	}
}

#[cfg(feature = "actix")]
mod actix_impl {
	use super::*;
	use actix_web::{HttpMessage, HttpRequest};

	impl Context for HttpRequest {
		fn get(&self, key: &str) -> Option<LogValue> {
			match key {
				"user_id" => self.extensions().get::<i32>().map(|i| LogValue::Display(Box::new(*i))),
				_ => None,
			}
		}
	}
}

#[cfg(feature = "axum")]
mod axum_impl {
	use super::*;
	use axum::http::Request;

	impl<B> Context for Request<B> {
		fn get(&self, key: &str) -> Option<LogValue> {
			match key {
				"user_id" => self.extensions().get::<i32>().map(|i| LogValue::Display(Box::new(*i))),
				_ => None,
			}
		}
	}
}
