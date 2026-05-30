use std::time::SystemTime;

use xxhash_rust::xxh3::xxh3_64_with_seed;

use super::{Result, RouteKind, Storable};
use crate::util::compress;

pub struct CompressedPaste {
	pub(super) slug:    String,
	pub(super) hash:    u64,
	pub(super) content: Vec<u8>,
}

impl CompressedPaste {
	pub fn into_content(self) -> Vec<u8> { self.content }

	pub fn slug(&self) -> String { self.slug.clone() }
}

impl Storable for CompressedPaste {
	fn id(&self) -> &str { &self.slug }

	fn kind(&self) -> RouteKind { RouteKind::Paste }

	fn hash(&self) -> Option<i64> { Some(self.hash as i64) }

	fn content(&self) -> Option<&[u8]> { Some(&self.content) }
}

pub struct PasteRecord {
	pub slug:    String,
	pub content: Vec<u8>,
}

pub struct PasteInput(PasteInputInner);

struct PasteInputInner {
	content: String,
	hash:    u64,
}

impl PasteInput {
	pub fn hash(&self) -> u64 { self.0.hash }

	pub fn content(&self) -> &str { &self.0.content }

	pub fn try_into_compressed_paste(
		self,
		slug: String,
	) -> Result<CompressedPaste> {
		let compressed = compress(self.0.content)?;
		Ok(CompressedPaste {
			slug,
			hash: self.0.hash,
			content: compressed,
		})
	}
}

impl From<String> for PasteInput {
	fn from(value: String) -> Self {
		let hash = xxh3_64_with_seed(value.as_bytes(), 0);
		Self(PasteInputInner {
			content: value,
			hash,
		})
	}
}
