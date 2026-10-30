use core::f64::math::ceil;
use std::char;
use std::fmt::{self, Display, Formatter};
use std::path::Path;
use std::random::{DefaultRandomSource, RandomSource, random};
use std::str::FromStr;

use rslog::{attrs, debug};

use super::{Error, Result};
use crate::config::logging::{RANDINT_COUNT_KEY, SLUG_KEY, SLUG_LEN_KEY};
use crate::util::normalize_http;

/// list of characters accepted in slug paths
const SLUG_BASE: u64 = SLUG_SPACE.len() as u64;
const SLUG_SPACE: &[u8] =
	b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

const BYTES_PER_WORD: usize = size_of::<u64>();
const DIGITS_PER_WORD: usize = 8;

/// default shell fired by the generator/shotgun
pub const SHOTGUN_SHELL: [SubBatch; 2] =
	[SubBatch { len: 4, qty: 11 }, SubBatch { len: 5, qty: 4 }];

/// Total number of pellets in [`SHOTGUN_SHELL`]
pub const SHOTGUN_ATTEMPTS: usize = {
	let mut total = 0;
	let mut i = 0;
	while i < SHOTGUN_SHELL.len() {
		total += SHOTGUN_SHELL[i].qty;
		i += 1;
	}
	total
};

/// Receptacle that can be fired to see what sticks
pub type ShotgunShell = [SubBatch];
/// Number of pellets of a given size in the shell
struct SubBatch {
	len: usize,
	qty: usize,
}
/// What the shotgun produces, aka multiple small candidates
/// striving to be a fully fledged slug. Candidates are split
/// by `/` as the character isn't allowed in slug urls.
type Pellets = String;

const FORBIDDEN_CHARS: [char; 9] =
	['.', '?', '&', '/', '#', '\\', '"', '%', '^'];

#[derive(Debug)]
pub struct Slug(Box<str>);
impl Slug {
	pub(crate) fn from_validated(raw: Box<str>) -> Self { Self(raw) }

	pub fn as_str(&self) -> &str { &self.0 }

	pub fn parse(raw: impl Into<Box<str>>, cap: usize) -> Result<Self> {
		let s = raw.into();
		if s.len() > cap {
			return Err(Error::InvalidSlug {
				reason: format!(
					"exceeds maximum allowed length of {cap} bytes (got {})",
					s.len()
				),
			});
		}
		if let Some(c) = s.chars().find(|c| FORBIDDEN_CHARS.contains(c)) {
			return Err(Error::InvalidSlug {
				reason: format!("contains undesirable character: '{c}'"),
			});
		}
		let normalized = normalize_http(Path::new(&*s));
		let normalized = normalized.to_string_lossy();
		if normalized.len() != s.len() {
			return Err(Error::InvalidSlug {
				reason: format!(
					"would be mangled by path normalization (`{s}` -> `{}`)",
					&normalized
				),
			});
		}
		Ok(Self(s))
	}
}

impl Display for Slug {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result { f.write_str(&self.0) }
}

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
/// The query returns the slug that got picked (the first one free).
/// Since slugs are short and cheap to produce, this introduces
/// a small fixed cost to minimize querying and memory usage.
///
/// With a default of 4char slugs, there are ~15M possible slugs (N)
/// If the service expects a 25k uploads/day average, the database should
/// hold about ~9M entries at any given point during the year (Y) which
/// means the load (L) is about ~62% of all possible values(Y/N).
/// This means there is about a ~62% chance that a single random slug
/// hits a collision. But by trying multiple ones, a shotgun attempt
/// trying 16 candidates at once only has a ~0.045% chance that all 16
/// candidates already exist (P(16)=0.00045).
///
/// A failure rate of ~0.05% means eleven failures/day on average (not good
/// enough). So the middle ground to the middle ground is:
/// - first try 11 slugs of length=4
/// - if ALL are taken, then try 4 more of length=5
/// They're bit more lengthy/less user-friendly, but just look at
/// those numbers:
/// - 25k daily requests: ~0.5% become to 5-byte slugs (the first 11 failed)
/// - for that many daily requests, probability of exhausting all 15 candidates
///   is ~1 failure every 5 trillion years (yeah...)
/// - there would need to be ~78k daily requests (~28.5 million/year) before
///   reaching an average of 1 dropped upload per year, assuming 1-year
///   retention
///
/// And remember those numbers are just for unique uploads. Deduplication of
/// existing slugs doesn't add up toward that total.
pub struct SlugGenerator {}

impl SlugGenerator {
	fn digits(word: u64) -> impl Iterator<Item = char> {
		std::iter::successors(Some(word), |v| Some(v / SLUG_BASE))
			.take(DIGITS_PER_WORD)
			.map(|v| SLUG_SPACE[(v % SLUG_BASE) as usize] as char)
	}

	#[allow(clippy::cast_possible_truncation)]
	pub fn generate(shell: &ShotgunShell) -> Pellets {
		let lens = || shell.iter().flat_map(|b| std::iter::repeat_n(b.len, b.qty));

		let words = lens().map(|l| l.div_ceil(DIGITS_PER_WORD)).sum::<usize>();
		let capacity = lens().map(|l| l + 1).sum::<usize>().saturating_sub(1);

		let mut buf = vec![0u8; words * BYTES_PER_WORD];
		DefaultRandomSource.fill_bytes(&mut buf);

		let mut stream = buf
			.as_chunks::<BYTES_PER_WORD>()
			.0
			.iter()
			.map(|w| u64::from_le_bytes(*w));

		let mut out = String::with_capacity(capacity);
		lens().enumerate().for_each(|(i, len)| {
			if i > 0 {
				out.push('/');
			}
			out.extend(
				stream
					.by_ref()
					.take(len.div_ceil(DIGITS_PER_WORD))
					.flat_map(|word| Self::digits(word))
					.take(len),
			);
		});
		out
	}
}
