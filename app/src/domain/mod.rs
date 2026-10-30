use std::io::Write;

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
	Content, ContentPacked, ContentPlain, Entry, EntryPacked, EntryPlain,
	RecordPacked, RecordPlain,
};
pub use slug::{SHOTGUN_ATTEMPTS, SHOTGUN_SHELL, Slug, SlugGenerator};
pub use upload::{
	Naming, RedirectLimits, Resolved, UploadApproved, UploadPending, UploadReady,
};
