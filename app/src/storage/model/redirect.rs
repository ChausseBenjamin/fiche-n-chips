use std::sync::Arc;
use std::time::SystemTime;

use super::{Result, RouteKind, Storable};
use crate::storage::Storage;
use crate::util::{compress, decompress, SlugGenerator};

pub struct CompressedRedirect {
	pub(super) short:       String,
	pub(super) destination: Vec<u8>,
}

impl Storable for CompressedRedirect {
	fn id(&self) -> &str { &self.short }

	fn kind(&self) -> RouteKind { RouteKind::Redirect }

	fn hash(&self) -> Option<i64> { None }

	fn content(&self) -> Option<&[u8]> { Some(&self.destination) }

	fn created(&self) -> Option<SystemTime> { None }
}

pub struct RedirectInput(RedirectInputInner);

struct RedirectInputInner {
	short:       Option<String>,
	destination: String,
}

impl RedirectInput {
	pub fn new(destination: String, short: Option<String>) -> Self {
		Self(RedirectInputInner { short, destination })
	}

	pub fn destination(&self) -> &str { &self.0.destination }

	pub fn short(&self) -> Option<&str> { self.0.short.as_deref() }

	// Can't use trait as I need outside info
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

pub struct Redirect(RedirectInner);

impl Redirect {
	pub fn destination(&self) -> &str { &self.0.destination }
}

struct RedirectInner {
	short:       String,
	destination: String,
}

impl TryFrom<CompressedRedirect> for Redirect {
	type Error = super::Error;

	fn try_from(value: CompressedRedirect) -> Result<Self> {
		Ok(Self(RedirectInner {
			short:       value.short,
			destination: decompress(value.destination)?,
		}))
	}
}
