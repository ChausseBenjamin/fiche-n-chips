use std::time::SystemTime;

use self::{
	attr::*, context::*, handler::*, level::*, log_valuer::*, record::*,
};

mod attr;
mod context;
mod handler;
mod level;
mod log_valuer;
mod logger;
mod record;
