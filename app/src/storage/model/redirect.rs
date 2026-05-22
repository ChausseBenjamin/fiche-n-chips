use std::{sync::Arc, time::SystemTime};

use super::{
	super::{Error, Result},
	Route,
};
use crate::{
	storage::Storage,
	util::{compress, SlugGenerator},
};

/// Representation where the content remains compressed
pub struct CompRedirect {
	pub(super) short: String,
	pub(super) destination: Vec<u8>,
}

impl From<CompRedirect> for Route {
	fn from(value: CompRedirect) -> Self {
		Route::Redirect(value)
	}
}

struct RawRedirectInner {
	// shortened version of the url chosen by the user (aka ID)
	// If the user wants an auto-generated url, short->None
	short: Option<String>,
	destination: String,
}

/// Representation where the content hasn't been compressed
pub struct RawRedirect(RawRedirectInner);

impl RawRedirect {
	pub fn new(destination: String, short: Option<String>) -> Self {
		Self(RawRedirectInner { short, destination })
	}
	pub fn destination(&self) -> &str {
		&self.0.destination
	}
	pub fn short(&self) -> Option<&str> {
		match &self.0.short {
			Some(v) => todo!(),
			None => None,
		}
	}
	pub fn try_into_comp_redirect(
		self,
		db: Arc<Storage>,
		slugs: Arc<SlugGenerator>,
	) -> Result<CompRedirect> {
		let destination = compress(self.0.destination)?;
		let short = match self.0.short {
			Some(v) => v,
			None => slugs.gen_unique(&db)?,
		};
		Ok(CompRedirect { short, destination })
	}
}
