use core::f64::math::ceil;
use std::char;
use std::random::random;

use rslog::{attrs, debug};

use crate::storage::{self, Storage};

/// list of characters accepted in slug paths
const SLUG_SPACE: &[u8] =
	b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const SLUG_BASE: usize = SLUG_SPACE.len();

pub struct SlugGenerator {
	/// number of u64 values needed to compute a slug of size n
	/// cached in the struct to avoid computing a log2 every-time
	randint_count: usize,
	slug_len:      usize,
}

impl SlugGenerator {
	#[allow(
		clippy::cast_precision_loss,      // SLUG_BASE is a small constant (62)
		clippy::cast_possible_truncation, // ceil() result is tiny (< 5)
		clippy::cast_sign_loss            // ceil() always returns non-negative
	)]
	pub fn new(slug_len: u8) -> Self {
		let randint_count =
			ceil(f64::from(slug_len) * f64::log2(SLUG_BASE as f64) / 64.0) as usize;
		let slug_len: usize = slug_len as usize;
		Self {
			randint_count,
			slug_len,
		}
	}

	#[allow(clippy::cast_possible_truncation)]
	fn random(&self) -> String {
		// Maximum ammount of bits needed
		let mut entropy: Vec<_> = (0..self.randint_count)
			.into_iter()
			.map(|_| random::<u64>(..))
			.collect();
		let out: String = (0..self.slug_len)
			.into_iter()
			.map(|_| {
				// Simple fallback if for some reason, entropy runs out
				let current = entropy.last().copied().unwrap_or_else(|| {
					// Debug because it only slows down execution, more of a developer
					// info than something a user should worry about.
					debug(
						"slug creation lacked random data, creating more on the spot...",
						attrs!(
							"slug_len" => self.slug_len,
							"randint_count" => self.randint_count),
					);
					random::<u64>(..)
				});
				// Store the entropy as a character in the space
				let c = SLUG_SPACE[(current % SLUG_BASE as u64) as usize] as char;
				// Remove the data that was just used for the lastest char
				if let Some(v) = entropy.last_mut() {
					*v /= SLUG_BASE as u64;
				}
				// If the last integer ran out of entropy, pop it to access the next one
				// on the following iteration. Useless to pop if None, I skip the
				// function call by defaulting to 1
				if entropy.last().copied().unwrap_or(1) == 0 {
					entropy.pop();
				}
				c
			})
			.collect();
		out
	}

	pub async fn gen_unique(
		&self,
		db: &Storage,
	) -> Result<String, storage::Error> {
		let mut output = self.random();
		while db.queries.slug_exists(&output).await? {
			debug(
				"collision while generating a slug name",
				attrs!( "slug" => output),
			);
			output = self.random();
		}
		Ok(output)
	}
}
