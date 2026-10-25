use crate::util::StorageSize;

#[derive(Clone, Copy)]
pub struct PragmaCfg<'a> {
	pub name: &'a str,
	pub val:  &'a str,
}

impl PragmaCfg<'_> {
	pub(super) fn mem_cache(size: StorageSize) -> Self {
		let kib = size.get() / 1024;
		let cache_str = Box::leak(kib.to_string().into_boxed_str());
		Self {
			name: "cache_size",
			val:  cache_str,
		}
	}
}

#[rustfmt::skip]
pub(super) const PRAGMA_CONSTRAINTS: &[PragmaCfg] = &[
	PragmaCfg{name: "busy_timeout", val: "1000"},
	PragmaCfg{name: "temp_store",   val: "MEMORY"},
];
