pub const SLUG_BATCH_SIZE: usize = 5;

pub enum Name {
	Fixed(Slug),
	Batch([Slug; SLUG_BATCH_SIZE]),
}

impl From<Option<Slug>> for Name {
	fn from(value: Option<Slug>) -> Self {
		if let Some(name) = value {
			Self::Fixed(name)
		} else {
			todo!("generate from `app/src/util/slugs.rs`")
		}
	}
}
