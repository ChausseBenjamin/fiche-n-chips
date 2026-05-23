use std::io::{self, Read, Write};

use brotli::{CompressorWriter, Decompressor};

pub fn compress(src: String) -> Result<Vec<u8>, io::Error> {
	let mut out = Vec::new();
	{
		let mut w = CompressorWriter::new(&mut out, 4096, 9, 22);
		w.write_all(src.as_bytes())?;
	}
	Ok(out)
}

pub fn decompress(src: Vec<u8>) -> Result<String, io::Error> {
	let mut out = String::new();
	Decompressor::new(&src[..], 4096).read_to_string(&mut out)?;
	Ok(out)
}
