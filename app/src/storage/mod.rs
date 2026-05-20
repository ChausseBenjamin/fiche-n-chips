use std::{error::Error, iter, sync::Arc};

pub mod model;
mod statements;

use rslog::{attrs, debug, error, src};
use statements::Statements;
use turso::{Builder, Connection};

use crate::{config::Config, util::valid_filepath};

pub const DEFAULT_DB_NAME: &str = "store.db";

static DB_SCHEMA: &str = include_str!("schema.sql");

#[rustfmt::skip]
const PRAGMA_CONSTRAINTS: &[PragmaConstraint] = &[
	PragmaConstraint("busy_timeout", "1000"),
	PragmaConstraint("temp_store",   "MEMORY"),
];

#[derive(Clone, Copy)]
pub struct PragmaConstraint<'a>(&'a str, &'a str);

impl PragmaConstraint<'_> {
	fn new(name: &str, val: &str) -> Self {
		Self(name, val)
	}
}

pub struct Storage {
	connection: Connection,
	queries: Statements,
}

impl Storage {
	pub async fn setup(cfg: Arc<Config>) -> Result<Self, Box<dyn Error>> {
		// INITIAL SETUP:
		debug("setting up the database connection", src!());
		let db_path = valid_filepath(&cfg.management.db_path, DEFAULT_DB_NAME)?;
		let db = Builder::new_local(db_path.to_str().unwrap_or_else(|| {
			// Reaching this implies the user provided a custom path.
			// It's better to exit early than fallback to the default because
			// doing so will create an empty DB which can confuse the user. It
			// won't be immediately obvious to him (other than parusing logs
			error("failed to parse provided path", attrs!("input" => &db_path));
			panic!();
		}))
		.build()
		.await?;
		let connection = db.connect()?;

		// Configure main pragma rules
		let cache_val = cfg.management.db_cache.to_string();
		for p in PRAGMA_CONSTRAINTS
			.iter()
			.chain([PragmaConstraint::new("cache_size", &cache_val)].iter())
		{
			connection.pragma_update(p.0, p.1).await?;
		}

		let mut rows = connection.query("PRAGMA integrity_check", ()).await?;
		if let Some(row) = rows.next().await? {
			let msg: String = row.get(0)?;
			if msg != "ok" {
				return Err(format!("integrity check failed: {msg}").into());
			}
		}

		// FINAL TOUCHES:
		let queries = Statements::setup(&connection).await?;

		Ok(Self {
			connection,
			queries,
		})
	}
}
