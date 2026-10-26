use std::borrow::Cow;

use bytes::Bytes;
use xxhash_rust::xxh3::xxh3_64_with_seed as hash;

use super::{Error, Result};
use crate::util::{compress, decompress};

pub struct Plain(pub(super) Bytes);

impl Plain {
	pub(super) fn compress(self) -> Result<Brotli> {
		Ok(Brotli(Bytes::from(compress(&self.0)?)))
	}
}

impl AsRef<[u8]> for Plain {
	fn as_ref(&self) -> &[u8] { &self.0 }
}

impl TryFrom<&Brotli> for Plain {
	type Error = Error;

	fn try_from(comp: &Brotli) -> Result<Self> {
		Ok(Plain(Bytes::from(decompress(comp.0.as_ref())?)))
	}
}

pub struct Brotli(pub Bytes);

pub struct Checksum(pub u64);
impl Checksum {
	pub(super) fn of(bytes: &Bytes) -> Self { Self(hash(bytes, 0)) }
}
impl From<Checksum> for i64 {
	fn from(c: Checksum) -> Self { c.0.cast_signed() }
}
impl From<i64> for Checksum {
	fn from(c: i64) -> Self { Self(c.cast_unsigned()) }
}
