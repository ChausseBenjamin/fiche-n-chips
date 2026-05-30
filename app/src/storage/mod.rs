use std::sync::Arc;

mod error;
pub mod model;
mod pragma;
mod statements;

use const_format::formatcp;
pub use error::{Error, Result};
use model::{Reserved, RouteKind, Storable};
use rslog::{attrs, debug, error, src};
use statements::Statements;
use turso::{Builder, Connection, Value};

use crate::config::Config;
use crate::storage::pragma::{PragmaCfg, PRAGMA_CONSTRAINTS};
use crate::util::valid_filepath;

// Used if the user provides a path/dir but no filename
pub const DEFAULT_DB_NAME: &str = "store.db";

static DB_SCHEMA: &str = include_str!("schema.sql");

const DEL_MISSING_RESERVED: &str = formatcp!(
	"DELETE FROM routes WHERE kind = {} AND ID NOT IN",
	RouteKind::Reserved as u8
);

const INSERT_ROUTES_SAFE: &str =
	"INSERT OR IGNORE INTO routes (id, kind, hash, content) VALUES";

pub struct Storage {
	connection:  Connection,
	pub queries: Statements,
}

impl Storage {
	pub async fn setup(cfg: Arc<Config>) -> Result<Self> {
		// INITIAL SETUP:
		debug("setting up the database connection", src!());
		let db_path = valid_filepath(&cfg.management.db_path, DEFAULT_DB_NAME)?;
		let db_exists = db_path.exists();
		let path_str = db_path.to_str().ok_or_else(|| {
			error(
				"failed to parse provided path",
				attrs!("input" => db_path.display().to_string()),
			);
			Error::SetupErr("db path contains invalid UTF-8".to_string())
		})?;
		let db = Builder::new_local(path_str).build().await?;
		let conn = db.connect()?;

		if db_exists {
			// INTEGRITY CHECK:
			let mut rows = conn.query("PRAGMA integrity_check", ()).await?;
			if let Some(row) = rows.next().await? {
				let msg: String = row.get(0)?;
				if msg != "ok" {
					return Err(Error::SetupErr(format!(
						"integrity check failed: {msg}"
					)));
				}
			}

			// SCHEMA FINGERPRINT CHECK:
			let stored: i64 = {
				let mut rows = conn.query("PRAGMA user_version", ()).await?;
				rows.next().await?.map(|r| r.get(0)).transpose()?.unwrap_or(0)
			};

			if stored != Self::schema_fingerprint() {
				drop(conn);
				drop(db);
				let backup = db_path.with_file_name(format!(
					"{}-{}",
					std::time::SystemTime::now()
						.duration_since(std::time::UNIX_EPOCH)
						.unwrap_or_default()
						.as_nanos(),
					db_path.file_name().unwrap_or_default().to_string_lossy()
				));
				std::fs::rename(&db_path, &backup)?;
				return Box::pin(Self::setup(cfg)).await;
			}
		} else {
			conn.execute(DB_SCHEMA, ()).await?;
			conn.pragma_update("user_version", Self::schema_fingerprint()).await?;
		}

		// PRAGMA RULES:
		for p in PRAGMA_CONSTRAINTS
			.iter()
			.chain([PragmaCfg::mem_cache(cfg.management.db_cache)].iter())
		{
			conn.pragma_update(p.name, p.val).await?;
		}

		// FINAL TOUCHES:
		let queries = Statements::setup(&conn).await?;

		Ok(Self {
			connection: conn,
			queries,
		})
	}

	fn schema_fingerprint() -> i64 {
		let mut hash: u32 = 2_166_136_261;
		for b in DB_SCHEMA.bytes() {
			hash ^= u32::from(b);
			hash = hash.wrapping_mul(16_777_619);
		}
		hash.cast_signed().into()
	}

	pub async fn sync_reserved_routes(
		&self,
		paths: Vec<Reserved>,
	) -> Result<u64> {
		if paths.is_empty() {
			error(
				"no http paths are reserved",
				attrs!("desc" => "the server should accept/serve stuff but this indicates api endpoints aren't protected and no homepages are being hosted..."),
			);
			// Return Ok to remain resilient and serve content
			// Best we can do is log the error...
			return Ok(0);
		}

		let mut conn = self.connection.clone();
		let tx = conn.transaction().await?;
		let mut modified = 0;

		let safe_insert_cmd = format!(
			"{} {}",
			INSERT_ROUTES_SAFE,
			vec!["(?, ?, ?, ?)"; paths.len()].join(", ")
		);
		let safe_insert_args = paths
			.iter()
			.flat_map(|p| {
				vec![
					Value::from(p.id().to_string()),
					Value::from(p.kind() as u8),
					Value::from(p.hash()),
					Value::from(p.content()),
				]
			})
			.collect::<Vec<Value>>();
		modified += tx.execute(safe_insert_cmd, safe_insert_args).await?;

		let del_cmd = format!(
			"{} ({})",
			DEL_MISSING_RESERVED,
			vec!["?"; paths.len()].join(", ")
		);
		let del_args = paths
			.iter()
			.map(|p| Value::from(p.id().to_string()))
			.collect::<Vec<Value>>();
		modified += tx.execute(del_cmd, del_args).await?;

		tx.commit().await?;
		Ok(modified)
	}
}
