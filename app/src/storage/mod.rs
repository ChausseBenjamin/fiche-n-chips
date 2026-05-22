use std::sync::Arc;

mod error;
pub mod model;
mod pragma;
mod statements;

pub use error::{Error, Result};
use rslog::{attrs, debug, error, src};
use statements::Statements;
use turso::{Builder, Connection};

use crate::{
	config::Config,
	storage::pragma::{PragmaCfg, PRAGMA_CONSTRAINTS},
	util::valid_filepath,
};

// Used if the user provides a path/dir but no filename
pub const DEFAULT_DB_NAME: &str = "store.db";

static DB_SCHEMA: &str = include_str!("schema.sql");

#[derive(Clone)]
pub struct Storage {
	connection: Connection,
	pub queries: Statements,
}

impl Storage {
	pub async fn setup(cfg: Arc<Config>) -> Result<Self, Error> {
		// INITIAL SETUP:
		debug("setting up the database connection", src!());
		let db_path = valid_filepath(&cfg.management.db_path, DEFAULT_DB_NAME)?;
		let db_exists = db_path.exists();
		let db = Builder::new_local(db_path.to_str().unwrap_or_else(|| {
			// Reaching this implies the user provided a custom path.
			// It's better to exit early than fallback to the default because
			// doing so will create an empty DB which can confuse the user. It
			// won't be immediately obvious to him (other than parusing logs
			error(
				"failed to parse provided path",
				attrs!("input" => db_path.display().to_string()),
			);
			panic!();
		}))
		.build()
		.await?;
		let connection = db.connect()?;

		if !db_exists {
			connection.execute(DB_SCHEMA, ()).await?;
		} else {
			// INTEGRITY CHECK:
			let mut rows = connection.query("PRAGMA integrity_check", ()).await?;
			if let Some(row) = rows.next().await? {
				let msg: String = row.get(0)?;
				if msg != "ok" {
					return Err(Error::SetupErr(format!(
						"integrity check failed: {msg}"
					)));
				}
			}

			// VALIDATE SCHEMA:
			Self::validate_schema(&connection).await?;
		}

		// PRAGMA RULES:
		for p in PRAGMA_CONSTRAINTS
			.iter()
			.chain([PragmaCfg::mem_cache(cfg.management.db_cache)].iter())
		{
			connection.pragma_update(p.name, p.val).await?;
		}

		// FINAL TOUCHES:
		let queries = Statements::setup(&connection).await?;

		Ok(Self {
			connection,
			queries,
		})
	}

	async fn fetch_schema(conn: &Connection) -> Result<String, Error> {
		let mut rows = conn
			.query("SELECT sql FROM sqlite_master WHERE type='table'", ())
			.await?;
		let mut out = String::new();
		while let Some(row) = rows.next().await? {
			if let turso::Value::Text(sql) = row.get_value(0)? {
				out.push_str(&sql);
				out.push('\n');
			}
		}
		Ok(out)
	}

	async fn validate_schema(conn: &Connection) -> Result<(), Error> {
		let actual = Self::fetch_schema(conn).await?;
		if Self::normalize_sql(DB_SCHEMA) != Self::normalize_sql(&actual) {
			return Err(Error::SetupErr(
				"database schema does not match embedded schema".to_string(),
			));
		}
		Ok(())
	}

	fn normalize_sql(sql: &str) -> String {
		let mut s: String = sql
			.lines()
			.map(|l| {
				let l = if let Some(idx) = l.find("--") {
					&l[..idx]
				} else {
					l
				};
				l.trim()
			})
			.filter(|l| !l.is_empty())
			.collect::<Vec<_>>()
			.join(" ");
		s = s.to_lowercase();
		s = s.replace("create table sqlite_sequence(name,seq)", "");
		s = s.replace(';', "");
		let mut out = String::new();
		let mut prev_ws = false;
		for c in s.chars() {
			if c.is_whitespace() {
				if !prev_ws {
					out.push(' ');
					prev_ws = true;
				}
			} else {
				out.push(c);
				prev_ws = false;
			}
		}
		out.trim().to_string()
	}
}
