use std::borrow::Cow;
use std::path::{Path, PathBuf};

use crate::util::normalize_http;

macro_rules! embed {
	($path:literal, $variant:path) => {
		WebFile(WebFileInner {
			path:    Cow::Owned(normalize_http(Path::new($path))),
			content: $variant(Cow::Borrowed(include_bytes!(concat!(
				env!("OUT_DIR"),
				$path
			)))),
			mime:    Cow::Borrowed(mime_for_extension(Path::new($path))),
		})
	};
}

fn mime_for_extension(path: &Path) -> &'static str {
	match path.extension().and_then(|e| e.to_str()) {
		Some("html") => "text/html; charset=utf-8",
		Some("css") => "text/css; charset=utf-8",
		Some("svg") => "image/svg+xml",
		Some("ico") => "image/x-icon",
		Some("txt") => "text/plain; charset=utf-8",
		Some("xml") => "text/xml",
		Some("woff2") => "font/woff2",
		Some("json") => "text/javascript; charset=utf8",
		Some("webmanifest") => "application/manifest+json",
		Some("map") => "application/json",
		Some("wasm") => "application/wasm",
		Some("woff") => "font/woff",
		Some("png") => "image/png",
		Some("webp") => "image/webp",
		Some("jpg" | "jpeg") => "image/jpeg",
		Some("js" | "mjs") => "text/javascript; charset=utf-8",
		_ => "application/octet-stream",
	}
}

#[rustfmt::skip]
pub fn embedded_files() -> Vec<WebFile<'static>> {
  vec![
    embed!("/404.html",                       FileContents::Brotli),
    embed!("/favicon.svg",                    FileContents::Brotli),
    embed!("/index.html",                     FileContents::Brotli),
    embed!("/main.css",                       FileContents::Brotli),
    embed!("/misc/icons.svg",                 FileContents::Brotli),
    embed!("/robots.txt",                     FileContents::Raw),
    embed!("/sitemap.xml",                    FileContents::Raw),
    embed!("/misc/jetbrainsmono-latin.woff2", FileContents::Raw),
    embed!("/apple-touch-icon.png",           FileContents::Raw),
    embed!("/icon-192.png",                   FileContents::Raw),
    embed!("/icon-512.png",                   FileContents::Raw),
    embed!("/icon-maskable-512.png",          FileContents::Raw),
    embed!("/og-image.png",                   FileContents::Raw),
    embed!("/site.webmanifest",               FileContents::Raw),
  ]
}

pub struct WebFile<'a>(WebFileInner<'a>);

impl WebFile<'static> {
	pub fn new(path: PathBuf, content: Vec<u8>, compressed: bool) -> Self {
		WebFile(WebFileInner {
			mime:    Cow::Borrowed(mime_for_extension(&path)),
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

	pub fn mime(&self) -> &str { self.0.mime.as_ref() }

	pub(crate) fn content(&self) -> &FileContents<'_> { &self.0.content }
}

struct WebFileInner<'a> {
	path:    Cow<'a, Path>,
	content: FileContents<'a>,
	mime:    Cow<'a, str>,
}

pub enum FileContents<'a> {
	Raw(Cow<'a, [u8]>),
	Brotli(Cow<'a, [u8]>),
}
