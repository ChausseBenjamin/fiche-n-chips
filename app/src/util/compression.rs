use std::io::{self, Write};

use brotli::CompressorWriter;

pub fn compress(src: String) -> Result<Vec<u8>, io::Error> {
	let mut out = Vec::new();
	{
		let mut w = CompressorWriter::new(&mut out, 4096, 9, 22);
		w.write_all(src.as_bytes())?;
	}
	Ok(out)
}

pub fn decompress(src: Vec<u8>) -> Result<String, io::Error> {
	todo!()
}
