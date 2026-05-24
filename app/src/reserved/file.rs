use std::borrow::Cow;
use std::io;
use std::path::{Path, PathBuf};

use crate::util::{decompress, normalize_http};

macro_rules! embed {
	($path:literal, $variant:path) => {
		WebFile(WebFileInner {
			path:    Cow::Owned(normalize_http(Path::new($path))),
			content: $variant(Cow::Borrowed(include_bytes!(concat!(
				env!("OUT_DIR"),
				$path
			)))),
		})
	};
}

#[rustfmt::skip]
pub fn embedded_files() -> Vec<WebFile<'static>> {
  vec![
    embed!("/404.html",                       FileContents::Brotli),
    embed!("/favicon.ico",                    FileContents::Raw),
    embed!("/index.html",                     FileContents::Brotli),
    embed!("/main.css",                       FileContents::Brotli),
    embed!("/robots.txt",                     FileContents::Raw),
    embed!("/sitemap.xml",                    FileContents::Raw),
    embed!("/misc/icons.svg",                 FileContents::Brotli),
    embed!("/misc/jetbrainsmono-latin.woff2", FileContents::Raw),
  ]
}

pub struct WebFile<'a>(WebFileInner<'a>);

impl WebFile<'static> {
	pub fn new(path: PathBuf, content: Vec<u8>, compressed: bool) -> Self {
		WebFile(WebFileInner {
			path:    Cow::Owned(path),
			content: if compressed {
				FileContents::Brotli(Cow::Owned(content))
			} else {
				FileContents::Raw(Cow::Owned(content))
			},
		})
	}
}

impl WebFile<'_> {
	pub fn path(&self) -> &Path { &self.0.path }

	pub(crate) fn content(&self) -> &FileContents<'_> { &self.0.content }
}

struct WebFileInner<'a> {
	path:    Cow<'a, Path>,
	content: FileContents<'a>,
}

pub enum FileContents<'a> {
	Raw(Cow<'a, [u8]>),
	Brotli(Cow<'a, [u8]>),
}

impl FileContents<'_> {
	pub fn uncompressed(&self) -> Result<Cow<'_, [u8]>, io::Error> {
		Ok(match self {
			FileContents::Raw(v) => Cow::Borrowed(v.as_ref()),
			FileContents::Brotli(v) => {
				Cow::Owned(decompress(v.to_vec())?.into_bytes())
			}
		})
	}
}
