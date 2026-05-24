mod compression;
mod datetime;
mod paths;
mod size;
mod slugs;

pub use compression::*;
pub use datetime::*;
pub use paths::*;
pub use size::*;
pub use slugs::*;

use crate::ui::{self, UiString, BOLD, RED_FG};

pub fn sorry() -> UiString { ui::string("Sorry Eh...\n", &[BOLD, RED_FG]) }
