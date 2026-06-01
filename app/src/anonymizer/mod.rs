use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::random::random;

use xxhash_rust::xxh3::xxh3_64_with_secret;

use crate::util::valid_filepath;

const DEFAULT_SEED_FILENAME: &str = "seed";
const SEED_SIZE: usize = 192;

pub type DiscreteId = u64;

pub struct Anonimizer {
	seed: Box<[u8]>,
}

impl Anonimizer {
	pub fn new(seed_path: impl AsRef<Path>) -> Result<Self, Error> {
		let file = match valid_filepath(seed_path.as_ref(), DEFAULT_SEED_FILENAME) {
			Ok(f) => f,
			Err(e) if e.kind() == ErrorKind::NotFound => {
				let p = seed_path.as_ref().to_path_buf();
				if let Some(parent) = p.parent() {
					fs::create_dir_all(parent)?;
				}
				p
			}
			Err(e) => return Err(e),
		};

		let seed: Box<[u8]> = if let Ok(meta) = fs::metadata(&file) {
			if meta.len() < 64 {
				return Err(Error::new(ErrorKind::InvalidData, "seed too small"));
			}
			fs::read(&file)?.into_boxed_slice()
		} else {
			let seed = Self::generate_seed();
			if let Some(parent) = file.parent() {
				fs::create_dir_all(parent)?;
			}
			fs::write(&file, &seed)?;
			seed
		};

		Ok(Self { seed })
	}

	pub fn cloak(&self, input: &[u8]) -> u64 {
		xxh3_64_with_secret(input, &self.seed)
	}

	fn generate_seed() -> Box<[u8]> {
		(0..SEED_SIZE).map(|_| random::<u8>(..)).collect()
	}
}
