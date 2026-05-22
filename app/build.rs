use std::{env, fs, io::Write, path::Path};

use brotli::CompressorWriter;

fn main() {
	println!("cargo:rerun-if-changed=assets/");

	let out = env::var("OUT_DIR").unwrap();
	let out_dir = Path::new(&out);

	process_dir("assets", out_dir).unwrap();
}

fn process_dir(dir: &str, out_dir: &Path) -> std::io::Result<()> {
	fs::create_dir_all(out_dir)?;
	for entry in fs::read_dir(dir)? {
		let entry = entry?;
		let path = entry.path();
		let name = entry.file_name();

		if path.is_dir() {
			process_dir(path.to_str().unwrap(), &out_dir.join(&name))?;
		} else {
			let Some(name) = name.to_str() else { continue };
			let src = fs::read(&path)?;
			let bytes = match path.extension().and_then(|e| e.to_str()) {
				Some("html") => {
					let s = String::from_utf8(src).unwrap();
					let mut buf = Vec::new();
					min_html(&s, &mut buf);
					compress(&buf)
				}
				Some("css") => {
					let s = String::from_utf8(src).unwrap();
					let mut buf = Vec::new();
					min_css(&s, &mut buf);
					compress(&buf)
				}
				Some("svg") => {
					let s = String::from_utf8(src).unwrap();
					let mut buf = Vec::new();
					min_svg(&s, &mut buf);
					compress(&buf)
				}
				_ => compress(&src),
			};
			fs::write(out_dir.join(name), bytes)?;
		}
	}
	Ok(())
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
		lightningcss::stylesheet::StyleSheet::parse(src, Default::default())
			.unwrap();
	sheet.minify(Default::default()).unwrap();
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

fn min_svg<'a>(src: &str, out: &'a mut Vec<u8>) -> &'a [u8] {
	let cfg = minify_html::Cfg::new();
	out.clear();
	out.extend(minify_html::minify(src.as_bytes(), &cfg));
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
