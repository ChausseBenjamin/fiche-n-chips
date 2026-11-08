use std::path::Path;
use std::time::Duration;

use sqlx::query_file_scalar;
use sqlx::sqlite::SqliteAutoVacuum::Incremental;
use sqlx::sqlite::SqliteJournalMode::Wal;
use sqlx::sqlite::SqliteSynchronous::Normal;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tokio::sync::Mutex;

use super::{Error, Result, Storage};
use crate::config::Config;
use crate::util::valid_filepath;

/// Used when the user provides a path/directory but no filename.
pub const DEFAULT_DB_NAME: &str = "store.db";

impl Storage {
	/// Opens (or creates) the SQLite database, runs migrations, and captures
	/// the configured page size.
	pub async fn setup(cfg: &Config) -> Result<Self> {
		let db_path = valid_filepath(&cfg.mgmt.db_path, DEFAULT_DB_NAME)?;
		let db = SqlitePoolOptions::new()
			.max_connections(10)
			.acquire_timeout(Duration::from_secs(5))
			.connect_with(connect_options(cfg, &db_path))
			.await?;

		sqlx::migrate!("src/storage/migrations").run(&db).await?;

		let page_size = query_file_scalar!("src/storage/statements/page_size.sql")
			.fetch_one(&db)
			.await?;
		let page_size = u64::try_from(page_size)
			.map_err(|_| Error::Formatting("invalid page size".to_owned()))?;

		Ok(Self {
			db,
			shrink: Mutex::new(()),
			page_size,
		})
	}
}

/// Connection-level settings for a new pool.
fn connect_options(cfg: &Config, path: &Path) -> SqliteConnectOptions {
	SqliteConnectOptions::new()
		.filename(path)
		.auto_vacuum(Incremental)
		.synchronous(Normal)
		.journal_mode(Wal)
		.busy_timeout(Duration::from_secs(1))
		.pragma("temp_store", "MEMORY")
		.pragma("cache_size", (&cfg.mgmt.db_cache.get() / 1024).to_string())
		.foreign_keys(true)
		.create_if_missing(true)
}
