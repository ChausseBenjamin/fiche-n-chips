use std::collections::HashMap;
use std::time::SystemTime;

use crate::anonymizer::DiscreteId;

pub struct RateLimit {
	/// How many entries the user has uploaded since being added to the Map
	count: u64,
	/// At what time to clear the HashMap entry
	reset: SystemTime,
}

pub struct RateLimiter {
	data: HashMap<DiscreteId, RateLimit>,
}
