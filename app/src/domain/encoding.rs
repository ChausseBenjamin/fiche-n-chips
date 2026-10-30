use bytes::Bytes;
use xxhash_rust::xxh3::xxh3_64_with_seed as hash;

use super::{Error, Result};
use crate::util::{compress, decompress};

pub struct Plain(pub(super) Box<[u8]>);

impl Plain {
	pub(super) fn compress(self) -> Result<Brotli> {
		Ok(Brotli(compress(&self.0)?))
	}
}

impl AsRef<[u8]> for Plain {
	fn as_ref(&self) -> &[u8] { &self.0 }
}

pub struct Brotli(pub Box<[u8]>);

impl Brotli {
	pub fn decompress(&self) -> Result<Plain> { Ok(Plain(decompress(&self.0)?)) }

	pub(super) fn matches(&self, plain: &[u8]) -> Result<bool> {
		Ok(self.decompress()?.as_ref() == plain)
	}
}

impl AsRef<[u8]> for Brotli {
	fn as_ref(&self) -> &[u8] { &self.0 }
}

impl TryFrom<&Brotli> for Plain {
	type Error = Error;

	fn try_from(comp: &Brotli) -> Result<Self> { Ok(Plain(decompress(&comp.0)?)) }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Checksum(pub u64);
impl Checksum {
	pub(super) fn of(bytes: &[u8]) -> Self { Self(hash(bytes, 0)) }
}
impl From<Checksum> for i64 {
	fn from(c: Checksum) -> Self { c.0.cast_signed() }
}
impl From<i64> for Checksum {
	fn from(c: i64) -> Self { Self(c.cast_unsigned()) }
}
