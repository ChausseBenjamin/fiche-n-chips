#![allow(clippy::unwrap_used)]
use std::io::Write;
use std::path::Path;
use std::{env, fs};

use brotli::CompressorWriter;
use lightningcss::stylesheet::{MinifyOptions, ParserOptions};

/// Shared stylesheet that [`inline_stylesheet`] splices into every HTML page.
///
/// Pages reference it with an ordinary `<link rel="stylesheet" href="main.css"
/// data-build-inline>`: real markup that still renders when served straight
/// from disk, which the build then replaces with the file's contents. That
/// keeps one source of truth for the design tokens while shipping each page as
/// a single self-contained document, so its critical CSS can never paint after
/// the markup.
const INLINE_STYLESHEET: &str = "main.css";

fn main() {
	println!("cargo:rerun-if-changed=assets/");

	let out = env::var("OUT_DIR").unwrap();
	let out_dir = Path::new(&out);
	let stylesheet =
		fs::read_to_string(format!("assets/{INLINE_STYLESHEET}")).unwrap();

	process_dir("assets", out_dir, &stylesheet).unwrap();
}

fn process_dir(
	dir: &str,
	out_dir: &Path,
	stylesheet: &str,
) -> std::io::Result<()> {
	fs::create_dir_all(out_dir)?;
	for entry in fs::read_dir(dir)? {
		let entry = entry?;
		let path = entry.path();
		let name = entry.file_name();

		if path.is_dir() {
			process_dir(path.to_str().unwrap(), &out_dir.join(&name), stylesheet)?;
		} else {
			let Some(name) = name.to_str() else { continue };
			let src = fs::read(&path)?;
			let bytes = match path.extension().and_then(|e| e.to_str()) {
				Some("html") => {
					let s = String::from_utf8(src).unwrap();
					let mut buf = Vec::new();
					min_html(&inline_stylesheet(&s, stylesheet), &mut buf);
					compress(&buf)
				}
				Some("css") => {
					let s = String::from_utf8(src).unwrap();
					let mut buf = Vec::new();
					min_css(&s, &mut buf);
					compress(&buf)
				}
				Some("svg") => compress(&src),
				_ => src,
			};
			fs::write(out_dir.join(name), bytes)?;
		}
	}
	Ok(())
}

/// Swaps the `data-build-inline` `<link>` for the stylesheet's contents.
///
/// The markup stays a valid `<link>` so the source files can still be served
/// as-is; the build turns it into a `<style>` block so the page needs no extra
/// request. A link without the marker would be silently unstyled in production
/// (`/main.css` is not served), so fail the build instead of shipping it.
fn inline_stylesheet(src: &str, stylesheet: &str) -> String {
	let marker = format!(
		r#"<link rel="stylesheet" href="{INLINE_STYLESHEET}" data-build-inline>"#
	);
	if src.contains(&marker) {
		return src.replace(&marker, &format!("<style>\n{stylesheet}\n</style>"));
	}
	if src.contains(&format!("href=\"{INLINE_STYLESHEET}\"")) {
		panic!("HTML links {INLINE_STYLESHEET} without the `{marker}` marker");
	}
	src.to_owned()
}

fn min_html<'a>(src: &str, out: &'a mut Vec<u8>) -> &'a [u8] {
	let cfg = minify_html::Cfg {
		minify_css: true,
		minify_js: true,
		..Default::default()
	};
	out.clear();
	out.extend(minify_html::minify(src.as_bytes(), &cfg));
	&out[..]
}

fn min_css<'a>(src: &str, out: &'a mut Vec<u8>) -> &'a [u8] {
	let mut sheet =
		lightningcss::stylesheet::StyleSheet::parse(src, ParserOptions::default())
			.unwrap();
	sheet.minify(MinifyOptions::default()).unwrap();
	out.clear();
	out.extend(
		sheet
			.to_css(lightningcss::stylesheet::PrinterOptions {
				minify: true,
				..Default::default()
			})
			.unwrap()
			.code
			.as_bytes(),
	);
	&out[..]
}

fn compress(src: &[u8]) -> Vec<u8> {
	let mut out = Vec::new();
	{
		let quality = 11;
		let mut w = CompressorWriter::new(&mut out, 4096, quality, 22);
		w.write_all(src).unwrap();
	}
	out
}
