use std::{sync::Arc, time::SystemTime};

use super::{super::Result, RouteKind, Storable};
use crate::{
	storage::Storage,
	util::{compress, SlugGenerator},
};

pub struct CompressedRedirect {
	pub(super) short: String,
	pub(super) destination: Vec<u8>,
}

impl Storable for CompressedRedirect {
	fn id(&self) -> &str {
		&self.short
	}
	fn kind(&self) -> RouteKind {
		RouteKind::Redirect
	}
	fn hash(&self) -> Option<i64> {
		None
	}
	fn content(&self) -> Option<&[u8]> {
		Some(&self.destination)
	}
	fn created(&self) -> Option<SystemTime> {
		None
	}
}

struct RedirectInputInner {
	short: Option<String>,
	destination: String,
}

pub struct RedirectInput(RedirectInputInner);

impl RedirectInput {
	pub fn new(destination: String, short: Option<String>) -> Self {
		Self(RedirectInputInner { short, destination })
	}
	pub fn destination(&self) -> &str {
		&self.0.destination
	}
	pub fn short(&self) -> Option<&str> {
		self.0.short.as_deref()
	}
	pub async fn try_into_compressed_redirect(
		self,
		db: Arc<Storage>,
		slugs: Arc<SlugGenerator>,
	) -> Result<CompressedRedirect> {
		let destination = compress(self.0.destination)?;
		let short = match self.0.short {
			Some(v) => v,
			None => slugs.gen_unique(&db).await?,
		};
		Ok(CompressedRedirect { short, destination })
	}
}
