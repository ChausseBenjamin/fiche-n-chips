mod paste;
mod redirect;
mod reserved;
mod row;

pub use paste::{CompressedPaste, PasteInput, PasteRecord};
pub use redirect::{Redirect, RedirectInput};
pub use reserved::Reserved;
pub use row::{
	DeletedRoute, ExpiringRoute, RouteKind, RouteRow, RowEntry, Storable,
};

type Error = super::Error;
type Result<T> = super::Result<T>;
