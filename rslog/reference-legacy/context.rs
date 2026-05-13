use std::fmt::Display;

pub trait Context {
	fn get(&self, key: &str) -> Option<Box<dyn Display>>;
}

pub struct Background;

impl Context for Background {
	fn get(&self, _key: &str) -> Option<Box<dyn Display>> {
		None
	}
}

#[cfg(feature = "actix")]
mod actix_impl {
	use actix_web::{HttpMessage, HttpRequest};

	use super::*;

	impl Context for HttpRequest {
		fn get(&self, key: &str) -> Option<Box<dyn Display>> {
			match key {
				"user_id" => {
					self.extensions().get::<i32>().map(|i| Some(Box::new(log_value!(*i))))
				}
				_ => None,
			}
		}
	}
}

#[cfg(feature = "axum")]
mod axum_impl {
	use axum::http::Request;

	use super::*;

	impl<B> Context for Request<B> {
		fn get(&self, key: &str) -> Option<Box<dyn Display>> {
			match key {
				"user_id" => {
					self.extensions().get::<i32>().map(|i| Some(Box::new(log_value!(*i))))
				}
				_ => None,
			}
		}
	}
}
