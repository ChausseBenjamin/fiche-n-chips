use std::{env, fs, io::Write, path::Path};

fn main() {
	println!("cargo:rerun-if-changed=assets/");

	let out = env::var("OUT_DIR").unwrap();
	let out_dir = Path::new(&out);

	for entry in fs::read_dir("assets").unwrap() {
		let entry = entry.unwrap();
		let path = entry.path();
		let name = entry.file_name();
		let name = name.to_str().unwrap();

		let src = fs::read(&path).unwrap();
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
			_ => compress(&src),
		};
		fs::write(out_dir.join(name), bytes).unwrap();
	}
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
		lightningcss::stylesheet::StyleSheet::parse(src, Default::default()).unwrap();
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

fn compress(src: &[u8]) -> Vec<u8> {
	let mut out = Vec::new();
	{
		let mut w = brotli::CompressorWriter::new(&mut out, 4096, 9, 22);
		w.write_all(src).unwrap();
	}
	out
}
