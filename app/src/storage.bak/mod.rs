use std::sync::Arc;

mod error;
pub mod model;
mod pragma;
mod statements;

pub use error::{Error, Result};
use model::{ExpiringRoute, Reserved};
use rslog::{attrs, debug, error, info, src, warn};
use statements::Statements;
use turso::{Builder, Connection};

use crate::config::Config;
use crate::config::logging::{
	BACKUP_KEY, COUNT_KEY, CURRENT_KEY, DESC_KEY, ERR_KEY, EXISTS_KEY, NAME_KEY,
	PATH_KEY, STORED_KEY, VALUE_KEY,
};
use crate::storage::pragma::{PRAGMA_CONSTRAINTS, PragmaCfg};
use crate::util::valid_filepath;

// Used if the user provides a path/dir but no filename
pub const DEFAULT_DB_NAME: &str = "store.db";

static DB_SCHEMA: &str = include_str!("sql/00-schema.sql");

pub struct Storage {
	connection:  Connection,
	pub queries: Statements,
	shrink_lock: tokio::sync::Mutex<()>,
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
		let db =
			Builder::new_local(path_str).experimental_vacuum(true).build().await?;
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
			shrink_lock: tokio::sync::Mutex::new(()),
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

		self.queries.sync_reserved(&paths).await
	}

	pub async fn shrink_to_fit(
		&self,
		max_bytes: u64,
	) -> Result<Vec<ExpiringRoute>> {
		// Inserts spawn a shrink each time, so several of these can overlap.
		// Without serializing them, each one observes the same over-budget
		// state and deletes its own tranche.
		let _guard = self.shrink_lock.lock().await;

		let mut all_deleted = Vec::new();

		if self.queries.db_size().await?.live() <= max_bytes {
			return Ok(all_deleted);
		}

		let mut conn = self.connection.clone();
		let tx = conn.transaction().await?;

		loop {
			let batch = self.queries.delete_oldest_batch().await?;
			if batch.is_empty() {
				break;
			}
			all_deleted.extend(batch);

			// Stop as soon as we are back under the cap instead of draining
			// every redirect and paste.
			if self.queries.db_size().await?.live() <= max_bytes {
				break;
			}
		}

		tx.commit().await?;

		// Deletes free pages but leave `page_count` unchanged, so the file on
		// disk stays large until vacuumed. The deletes are already committed,
		// so failing here should not be reported as a failed shrink.
		if self.queries.db_size().await?.physical() > max_bytes {
			if let Err(e) = self.queries.vacuum().await {
				warn(
					"failed to reclaim disk space after shrinking database",
					attrs!(ERR_KEY => e),
				);
			}
		}

		Ok(all_deleted)
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
