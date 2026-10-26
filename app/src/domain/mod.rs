use std::io::Write;
use std::marker::PhantomData;

use brotli::CompressorWriter;
use bytes::Bytes;
use xxhash_rust::xxh3::xxh3_64_with_seed as hash;

use crate::domain::encoding::AsPlainRef;
use crate::util::{compress, decompress};

mod brief;
mod encoding;
mod error;
mod record;
mod slug;
mod upload;

pub use brief::{Brief, RouteKind};
pub use encoding::{Brotli, Checksum, Plain};
pub use error::{Error, Result};
pub use record::{Content, Entry, Existing, Record};
pub use slug::Slug;
pub use upload::Upload;

pub struct Blob<E> {
	bytes:    E,
	checksum: Checksum,
}
impl<E: AsPlainRef> Blob<E> {
	pub fn matches(&self, plain: &[u8]) -> Result<bool> {
		Ok(self.bytes.as_plain_ref()? == plain)
	}
}
impl Blob<Plain> {
	pub fn new(bytes: &[u8]) -> Self {
		Self {
			bytes:    Plain(Bytes::from(bytes)),
			checksum: Checksum::of(&bytes),
		}
	}
}
impl Blob<Brotli> {
	pub fn from_compressed(bytes: Bytes, checksum: Checksum) -> Self {
		Self {
			bytes: Brotli(bytes),
			checksum,
		}
	}
}
