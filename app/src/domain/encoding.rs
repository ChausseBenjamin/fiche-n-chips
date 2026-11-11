use std::borrow::Cow;

use bytes::Bytes;
use xxhash_rust::xxh3::xxh3_64_with_seed as hash;

use super::{Error, Result};
use crate::util::{compress, decompress};

pub trait AsPlainRef {
	fn as_plain_ref(&self) -> Result<Cow<'_, [u8]>>;
}

pub struct Plain(pub(super) Bytes);
impl TryFrom<&Brotli> for Plain {
	type Error = Error;

	fn try_from(comp: &Brotli) -> Result<Self> {
		Ok(Plain(Bytes::from(decompress(comp.0.as_ref())?)))
	}
}
impl AsPlainRef for Plain {
	fn as_plain_ref(&self) -> Result<Cow<'_, [u8]>> {
		Ok(Cow::Borrowed(self.0.as_ref()))
	}
}

pub struct Brotli(pub Bytes);
impl TryFrom<&Plain> for Brotli {
	type Error = Error;

	fn try_from(plain: &Plain) -> Result<Self> {
		Ok(Brotli(Bytes::from(compress(&plain.0)?)))
	}
}
impl AsPlainRef for Brotli {
	fn as_plain_ref(&self) -> Result<Cow<'_, [u8]>> {
		Ok(Cow::Owned(decompress(self.0.as_ref())?))
	}
}

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
