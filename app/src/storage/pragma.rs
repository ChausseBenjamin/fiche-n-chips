#[derive(Clone, Copy)]
pub struct PragmaCfg<'a> {
	pub name: &'a str,
	pub val: &'a str,
}

impl<'a> PragmaCfg<'a> {
	pub(super) fn new(name: &'a str, val: &'a str) -> Self {
		Self { name, val }
	}
	pub(super) fn mem_cache(mb: u16) -> Self {
		// Ok to leak -> this function only ever runs once during setup
		let cache_str = Box::leak(mb.to_string().into_boxed_str());
		Self {
			name: "cache_size",
			val: cache_str,
		}
	}
}

#[rustfmt::skip]
pub(super) const PRAGMA_CONSTRAINTS: &[PragmaCfg] = &[
	PragmaCfg{name: "busy_timeout", val: "1000"},
	PragmaCfg{name: "temp_store",   val: "MEMORY"},
];
