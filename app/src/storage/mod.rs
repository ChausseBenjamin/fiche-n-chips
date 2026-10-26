use std::sync::Arc;
use std::time::Duration;

use sqlx::sqlite::SqliteAutoVacuum::Incremental;
use sqlx::sqlite::SqliteJournalMode::Wal;
use sqlx::sqlite::SqliteSynchronous::Normal;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};

use crate::config::Config;

mod error;
mod primitives;

pub use error::{Error, Result};
pub use primitives::RowKind;

pub struct Storage {
	cfg: Arc<Config>,
	db:  SqlitePool,
}

impl Storage {
	pub async fn setup(cfg: Arc<Config>) -> Result<Self> {
		let opts = SqliteConnectOptions::new()
			.filename(&cfg.mgmt.db_path)
			.auto_vacuum(Incremental)
			.synchronous(Normal)
			.journal_mode(Wal)
			.busy_timeout(Duration::from_secs(1))
			.pragma("temp_store", "MEMORY")
			.pragma("cache_size", (&cfg.mgmt.db_cache.get() / 1024).to_string())
			.foreign_keys(true)
			.create_if_missing(true);
		let db = SqlitePool::connect_with(opts).await?;
		sqlx::migrate!("src/storage/migrations").run(&db).await?;
		Ok(Self { cfg, db })
	}
}
