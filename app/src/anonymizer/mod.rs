use std::io::Error;
use std::path::Path;
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
		let file = valid_filepath(seed_path, DEFAULT_SEED_FILENAME)?;
		let empty_file = true; // TODO...
		if empty_file {
			let seed = Self::generate_seed();
			todo!("Write seed to file");
			Ok(Self { seed })
		} else {
			todo!("Read the seed into Self")
		}
	}

	pub fn cloak(&self, input: &[u8]) -> u64 {
		xxh3_64_with_secret(input, &self.seed)
	}

	fn generate_seed() -> Box<[u8]> {
		(0..SEED_SIZE).into_iter().map(|_| random::<u8>(..)).collect()
	}
}
