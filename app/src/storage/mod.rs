use std::sync::Arc;

mod error;
pub mod model;
mod pragma;
mod statements;

use const_format::formatcp;
pub use error::{Error, Result};
use model::{ExpiringRoute, Reserved, RouteKind, Storable};
use rslog::{attrs, debug, error, info, src, warn};
use statements::Statements;
use turso::{Builder, Connection, Value};

use crate::config::Config;
use crate::config::logging::{
	BACKUP_KEY, COUNT_KEY, CURRENT_KEY, DESC_KEY, ERR_KEY, EXISTS_KEY, NAME_KEY,
	PATH_KEY, STORED_KEY, VALUE_KEY,
};
use crate::storage::pragma::{PRAGMA_CONSTRAINTS, PragmaCfg};
use crate::util::{from_unix_timestamp, valid_filepath};

// Used if the user provides a path/dir but no filename
pub const DEFAULT_DB_NAME: &str = "store.db";

static DB_SCHEMA: &str = include_str!("sql/00-schema.sql");

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
		let db_path = valid_filepath(&cfg.management.db_path, DEFAULT_DB_NAME)?;
		let path_str = db_path.to_str().ok_or_else(|| {
			error(
				"failed to parse provided path",
				attrs!(PATH_KEY => db_path.display().to_string()),
			);
			Error::Setup("db path contains invalid UTF-8".to_string())
		})?;
		let db_exists = db_path.exists();
		info(
			"opening database",
			attrs!(
				PATH_KEY => path_str.to_owned(),
				EXISTS_KEY => db_exists),
		);
		let db = Builder::new_local(path_str).build().await?;
		let conn = db.connect()?;

		if db_exists {
			// INTEGRITY CHECK:
			let mut rows = conn.query("PRAGMA integrity_check", ()).await?;
			if let Some(row) = rows.next().await? {
				let msg: String = row.get(0)?;
				if msg != "ok" {
					return Err(Error::Setup(format!("integrity check failed: {msg}")));
				}
				debug("integrity check passed", src!());
			}

			// SCHEMA FINGERPRINT CHECK:
			let stored: i64 = {
				let mut rows = conn.query("PRAGMA user_version", ()).await?;
				rows.next().await?.map(|r| r.get(0)).transpose()?.unwrap_or(0)
			};
			let current = Self::schema_fingerprint();
			debug(
				"schema fingerprint",
				attrs!(
					STORED_KEY => stored,
					CURRENT_KEY => current),
			);

			if stored != current {
				warn(
					"database schema mismatch — backing up and recreating",
					attrs!(
            STORED_KEY => stored,
            CURRENT_KEY => current),
				);
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
				info(
					"database backed up",
					attrs!(BACKUP_KEY => backup.display().to_string()),
				);
				std::fs::rename(&db_path, &backup)?;
				info("retrying database setup", src!());
				return Box::pin(Self::setup(cfg)).await;
			}
		} else {
			info("creating new database", src!());
			conn.execute(DB_SCHEMA, ()).await?;
			conn.pragma_update("user_version", Self::schema_fingerprint()).await?;
		}

		// PRAGMA RULES:
		for p in PRAGMA_CONSTRAINTS
			.iter()
			.chain([PragmaCfg::mem_cache(cfg.management.db_cache)].iter())
		{
			conn.pragma_update(p.name, p.val).await?;
			debug(
				"applied pragma",
				attrs!(NAME_KEY => p.name, VALUE_KEY => p.val),
			);
		}

		// FINAL TOUCHES:
		let queries = Statements::setup(&conn).await?;

		info("database setup complete", src!());

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
				attrs!(
					DESC_KEY => "the server should accept/serve stuff but getting this error indicates api endpoints aren't protected and no homepages are being hosted...",
				),
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

	pub async fn shrink_to_fit(
		&self,
		max_bytes: u64,
	) -> Result<Vec<ExpiringRoute>> {
		let mut all_deleted = Vec::new();

		let current_size = self.get_db_size().await?;
		if current_size <= max_bytes {
			return Ok(all_deleted);
		}

		let mut conn = self.connection.clone();
		let tx = conn.transaction().await?;

		loop {
			let mut rows = tx
				.query(
					"DELETE FROM routes WHERE kind != 0 ORDER BY created_on ASC LIMIT 10 \
           RETURNING id, kind, created_on",
					(),
				)
				.await?;

			let mut batch = Vec::new();
			while let Some(row) = rows.next().await? {
				batch.push(ExpiringRoute {
					id:      row.get(0)?,
					kind:    (row.get::<i64>(1)?).try_into()?,
					created: from_unix_timestamp(row.get::<i64>(2)?),
				});
			}

			if batch.is_empty() {
				break;
			}

			all_deleted.extend(batch);
		}

		tx.commit().await?;

		Ok(all_deleted)
	}

	async fn get_db_size(&self) -> Result<u64> {
		let mut rows = self.connection.query("PRAGMA page_count", ()).await?;
		let row = rows
			.next()
			.await?
			.ok_or_else(|| Error::Setup("failed to query page_count".into()))?;
		let page_count: i64 = row.get(0)?;

		let mut rows = self.connection.query("PRAGMA page_size", ()).await?;
		let row = rows
			.next()
			.await?
			.ok_or_else(|| Error::Setup("failed to query page_size".into()))?;
		let page_size: i64 = row.get(0)?;

		Ok(page_count.unsigned_abs() * page_size.unsigned_abs())
	}

	pub fn spawn_shrink(self: &Arc<Self>, max_bytes: u64) {
		let this = Arc::clone(self);
		tokio::spawn(async move {
			match this.shrink_to_fit(max_bytes).await {
				Ok(deleted) => {
					if deleted.is_empty() {
						debug("db is within size limit after insert", src!());
					} else {
						info(
							"freed space by deleting old entries after insert",
							attrs!(COUNT_KEY => deleted.len()),
						);
					}
				}
				Err(e) => {
					error("failed to shrink db after insert", attrs!(ERR_KEY => e));
				}
			}
		});
	}
}
