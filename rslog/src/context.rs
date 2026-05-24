pub trait Context {
	fn get<T: Clone + Send + Sync + 'static>(&self) -> Option<T>;
}

pub struct Background;

impl Context for Background {
	fn get<T: Clone + Send + Sync + 'static>(&self) -> Option<T> { None }
}

#[cfg(feature = "actix")]
mod actix_impl {
	use actix_web::{HttpMessage, HttpRequest};

	use super::Context;

	impl Context for HttpRequest {
		fn get<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
			// extensions() returns Ref<'_, Extensions> — must clone out before guard
			// drops.
			self.extensions().get::<T>().cloned()
		}
	}

	#[cfg(test)]
	mod tests {
		use actix_web::test::TestRequest;

		use super::*;

		#[derive(Clone, Debug, PartialEq)]
		struct UserId(pub u64);

		#[test]
		fn returns_none_when_not_set() {
			let req = TestRequest::default().to_http_request();
			assert_eq!(req.get::<UserId>(), None);
		}

		#[test]
		fn returns_value_when_set() {
			let req = TestRequest::default().to_http_request();
			req.extensions_mut().insert(UserId(42));
			assert_eq!(req.get::<UserId>(), Some(UserId(42)));
		}
	}
}

#[cfg(feature = "axum")]
mod axum_impl {
	use axum::http::Request;

	use super::Context;

	impl<B> Context for Request<B> {
		fn get<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
			// extensions() returns &Extensions directly — no guard, clone is still
			// required to match the trait's owned return type.
			self.extensions().get::<T>().cloned()
		}
	}

	#[cfg(test)]
	mod tests {
		use axum::http::Request;

		use super::*;

		#[derive(Clone, Debug, PartialEq)]
		struct UserId(pub u64);

		#[test]
		fn returns_none_when_not_set() {
			let req = Request::new(());
			assert_eq!(req.get::<UserId>(), None);
		}

		#[test]
		fn returns_value_when_set() {
			let mut req = Request::new(());
			req.extensions_mut().insert(UserId(42));
			assert_eq!(req.get::<UserId>(), Some(UserId(42)));
		}
	}
}
