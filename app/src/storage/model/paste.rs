use std::time::SystemTime;

use xxhash_rust::xxh3::{xxh3_64, xxh3_64_with_seed};

use super::{
	super::{Error, Result},
	Route,
};
use crate::util::compress;

/// Representation where the content remains compressed
pub struct CompPaste {
	pub(super) slug: String,
	pub(super) hash: u64, // format used by xxh3
	pub(super) content: Vec<u8>,
}

impl From<CompPaste> for Route {
	fn from(value: CompPaste) -> Self {
		Route::Paste(value)
	}
}

pub struct MatchingPaste {
	pub slug: String,
	pub content: Vec<u8>,
}

/// Every received paste starts off as a String
/// and each will be need to be hashed for both:
/// - check against the DB for matching files
/// - insertion in the DB
pub struct HashedPaste(HashedPasteInner);

struct HashedPasteInner {
	content: String,
	hash: u64,
}

impl HashedPaste {
	pub fn hash(&self) -> u64 {
		self.0.hash
	}
	pub fn content(&self) -> &str {
		&self.0.content
	}
	pub fn try_into_comp_paste(self, slug: String) -> Result<CompPaste> {
		let compressed = compress(self.0.content)?;
		Ok(CompPaste {
			slug,
			hash: self.0.hash,
			content: compressed,
		})
	}
}

impl From<String> for HashedPaste {
	fn from(value: String) -> Self {
		let hash = xxh3_64_with_seed(value.as_bytes(), 0);
		Self(HashedPasteInner {
			content: value,
			hash,
		})
	}
}
