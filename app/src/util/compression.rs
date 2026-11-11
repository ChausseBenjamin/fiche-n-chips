use std::io::{self, Read, Write};

use brotli::{CompressorWriter, Decompressor};

const WINDOW: u32 = 22; // 4MiB rolling window
const QUALITY: u32 = 9;
const BUF_SIZE: usize = 4096;

pub fn compress(src: &[u8]) -> Result<Vec<u8>, io::Error> {
	let mut out = Vec::new();
	{
		let mut w = CompressorWriter::new(&mut out, BUF_SIZE, QUALITY, WINDOW);
		w.write_all(src)?;
	}
	Ok(out)
}

pub fn decompress(src: &[u8]) -> Result<Vec<u8>, io::Error> {
	let mut out = Vec::new();
	Decompressor::new(src, 4096).read_to_end(&mut out)?;
	Ok(out)
}
