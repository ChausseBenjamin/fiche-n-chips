#[repr(u8)]
pub enum RouteKind {
	Reserved = 0,
	Redirect = 1,
	Paste    = 2,
}
