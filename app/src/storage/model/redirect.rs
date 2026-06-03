use std::path::Path;
use std::sync::Arc;

use super::{Result, RouteKind, Storable};
use crate::storage::Storage;
use crate::util::{compress, decompress, normalize_http, SlugGenerator};

fn validate_slug(slug: &str) -> Result<()> {
	let undesirable = ['.', '?', '&', '/'];
	if let Some(c) = slug.chars().find(|c| undesirable.contains(c)) {
		return Err(super::Error::Parse(format!(
			"slug contains undesirable character: '{c}'"
		)));
	}
	let normalized = normalize_http(Path::new(slug));
	let normalized = normalized.to_string_lossy();
	if normalized.len() != slug.len() {
		return Err(super::Error::Parse(format!(
			"slug would be mangled by path normalization ('{slug}' -> '{normalized}')"
		)));
	}
	Ok(())
}

pub struct CompressedRedirect {
	pub(super) short:       String,
	pub(super) destination: Vec<u8>,
}

impl Storable for CompressedRedirect {
	fn id(&self) -> &str { &self.short }

	fn kind(&self) -> RouteKind { RouteKind::Redirect }

	fn hash(&self) -> Option<i64> { None }

	fn content(&self) -> Option<&[u8]> { Some(&self.destination) }
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

	// Can't use trait as I need outside info
	pub async fn try_into_compressed_redirect(
		self,
		db: &Storage,
		slugs: Arc<SlugGenerator>,
	) -> Result<CompressedRedirect> {
		let destination = compress(&self.0.destination)?;
		let short = match self.0.short {
			Some(v) => {
				validate_slug(&v)?;
				v
			}
			None => slugs.gen_unique(db).await?,
		};
		Ok(CompressedRedirect { short, destination })
	}
}

pub struct Redirect(RedirectInner);

impl Redirect {
	pub fn origin(&self) -> &str { &self.0.origin }

	pub fn destination(&self) -> &str { &self.0.destination }
}

struct RedirectInner {
	origin:      String,
	destination: String,
}

impl TryFrom<CompressedRedirect> for Redirect {
	type Error = super::Error;

	fn try_from(value: CompressedRedirect) -> Result<Self> {
		Ok(Self(RedirectInner {
			origin:      value.short,
			destination: decompress(&value.destination)?,
		}))
	}
}
