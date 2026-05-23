use std::io;

use crate::util::decompress;

macro_rules! embed {
	($path:literal) => {
		($path, include_bytes!(concat!(env!("OUT_DIR"), $path)))
	};
}

static EMBEDDED: &[(&str, &[u8])] = &[
	embed!("/404.html"),
	embed!("/favicon.ico"),
	embed!("/index.html"),
	embed!("/main.css"),
	embed!("/robots.txt"),
	embed!("/sitemap.xml"),
	embed!("/misc/icons.svg"),
	embed!("/misc/jetbrainsmono-latin.woff2"),
];

/// Wether the file was compressed during build or not
/// Ex: woff2 already uses brotli internally -> no need to use it twice
pub enum FileContents {
	Raw(Box<[u8]>),
	Brotli(Box<[u8]>),
}

pub struct WebFile {
	path: Box<str>,
	content: FileContents,
}

impl FileContents {
	pub fn uncompressed(&self) -> Result<Box<[u8]>, io::Error> {
		Ok(match self {
			FileContents::Raw(v) => v.clone(),
			FileContents::Brotli(comp) => {
				decompress(comp.to_vec())?.into_bytes().into_boxed_slice()
			}
		})
	}
}
