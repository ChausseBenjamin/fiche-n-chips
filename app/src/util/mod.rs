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

use crate::ui::{self, UiString, BOLD, RED_FG, YELLOW_FG, UNDERLINE, GREEN_FG, BLUE_FG};

pub const IDENT:    &[&str] = &[BOLD, YELLOW_FG];
pub const CONTACT:  &[&str] = &[BOLD, UNDERLINE];
pub const EMPHASIS: &[&str] = &[BLUE_FG];

pub fn sorry()   -> UiString { ui::string("Sorry Eh...\n", &[BOLD, RED_FG]) }
pub fn success() -> UiString { ui::string("cheers!", &[GREEN_FG]) }
