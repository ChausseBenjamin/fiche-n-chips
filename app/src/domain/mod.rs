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
pub use record::{
	ContentPacked, ContentPlain, Entry, RecordPending, RecordStored,
};
pub use slug::{SHOTGUN_SHELL, Slug, SlugGenerator};
pub use upload::{PackedUpload, UploadApproved, UploadPending, UploadReady};
