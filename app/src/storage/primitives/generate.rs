use core::f64::math::ceil;
use std::char;
use std::random::{DefaultRandomSource, RandomSource, random};

use rslog::{attrs, debug};

use crate::config::logging::{RANDINT_COUNT_KEY, SLUG_KEY, SLUG_LEN_KEY};
use crate::domain::Slug;
use crate::storage::{self, Storage};

/// list of characters accepted in slug paths
const SLUG_BASE: u64 = SLUG_SPACE.len() as u64;
const SLUG_SPACE: &[u8] =
	b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

const BYTES_PER_WORD: usize = size_of::<u64>();
const DIGITS_PER_WORD: usize = 8;

const SHOTGUN: usize = 16;

/// Computes a list of random, yet valid slug with a set-length
///
/// This was quite a unique problem to solve because I want to:
/// - minimize collisions when many possible slugs are taken
/// - not be susceptible to malicious actors trying to break insertion
///
/// In an ideal world, a sequence would minimize collisions.
/// But bad actors can forge redirects to upcoming values to
/// fuck with upcoming slugs (as those can be manually set).
///
/// Attempting to insert multiple times with a new random slug
/// makes it infeasible for bad actors to guess but is inefficient
/// as it requires multiple clones+memory allocations
/// (1backup if collisions happen, 1-2 allocation in sqlx+sqlite
/// during the insert)
///
/// So I came up with a middle ground: A shotgun approach
/// Multiple slugs are generated at once. The database query
/// is GIVEN (by value, not ref) the contents of an upload
/// (no duplicate copy) along with multiple slug candidates.
/// The query returns the slug that got picked. Since slugs are
/// short and cheap to produce, this introduces a small fixed cost
/// which minimizes querying and memory usage.
///
/// With a default of 4char slugs, there are ~15M possible slugs (N)
/// If the service expects a 25k uploads/day average, the database should
/// hold about ~9M entries at any given point during the year (Y) which
/// means the load (L) is about ~62% of all possible values(Y/N).
/// This means there is about a ~62% chance that a single random slug
/// hits a collision. But by trying multiple ones, a shotgun attempt
/// trying 16 candidates at once only has a ~0.045% chance that all 16
/// candidates already exist (P(16)=0.00045).
pub struct SlugGenerator {}

impl SlugGenerator {
	pub fn candidates(slug_len: u8) -> [Slug; SHOTGUN] {
		let size = usize::from(slug_len);
		let words = size.div_ceil(DIGITS_PER_WORD);
		let mut buf = vec![0u8; SHOTGUN * words * BYTES_PER_WORD];
		DefaultRandomSource.fill_bytes(&mut buf);
		let (chunks, _) = buf.as_chunks::<BYTES_PER_WORD>();
		std::array::from_fn(|i| Self::encode(size, &chunks[i * words..][..words]))
	}

	#[allow(clippy::cast_possible_truncation)]
	fn encode(size: usize, words: &[[u8; BYTES_PER_WORD]]) -> Slug {
		let mut out = String::with_capacity(size);
		out.extend(
			words
				.iter()
				.flat_map(|w| {
					let mut v = u64::from_le_bytes(*w);
					(0..DIGITS_PER_WORD).map(move |_| {
						let d = (v % SLUG_BASE) as usize;
						v /= SLUG_BASE;
						SLUG_SPACE[d] as char
					})
				})
				.take(size),
		);
		Slug::from_validated(out.into_boxed_str())
	}
}
