use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use sqlx::sqlite::SqliteAutoVacuum::Incremental;
use sqlx::sqlite::SqliteJournalMode::Wal;
use sqlx::sqlite::SqliteSynchronous::Normal;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use sqlx::{query_as, query_scalar};
use xxhash_rust::xxh3::xxh3_64_with_seed;

use crate::config::Config;
use crate::domain::{Brief, Novel, Record, RecordStored, Slug, UploadReady, UploadPending};

mod error;
mod primitives;

pub use error::{Error, Result};
use primitives::{Meta, Route, RowKind};

pub struct Storage {
	cfg: Arc<Config>,
	db:  SqlitePool,
  // TODO: Managing slug collisions is a Storage concern, move slug generation here
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

	pub async fn potential_duplicates(
		&self,
		upload: &UploadPending,
	) -> Result<Vec<RecordStored>> {
		let plain = upload.content();
		let hash = xxh3_64_with_seed(&plain, 0).cast_signed();
		query_as!(Route, r#"SELECT
          id AS "id!: _", kind AS "kind!: _", content AS "content?: _", created AS "created!: _"
          FROM routes WHERE hash = ?"#,
          hash,
        ).fetch_all(&self.db)
        .await?.into_iter().map(RecordStored::try_from).collect()
	}

	pub async fn get(&self, id: Slug) -> Result<RecordStored> {
		Ok(query_as!(Route,r#"SELECT
          id AS "id!: _", kind AS "kind!: _", content AS "content?: _", created AS "created!: _"
          FROM routes WHERE id = ?"#, id.as_str())
          .fetch_one(&self.db)
          .await?.map(RecordStored::try_from))
	}

	pub async fn get_brief(&self, slug: &Path) -> Result<Brief> {
		let id = slug.to_string_lossy();
		let meta = query_as!(
			Meta,
			r#"SELECT
          id AS "id!: _", kind AS "kind!: _", created AS "created!: _"
          FROM routes WHERE id = ? "#,
			id.as_ref(),
		)
		.fetch_one(&self.db)
		.await?;
		Ok(meta.into())
	}

	pub async fn exists(&self, id: Slug) -> Result<bool> {
		Ok(
			query_scalar!(
				r#"SELECT EXISTS(SELECT 1 FROM routes WHERE id = ?) AS "exists!: bool" "#,
				id.as_str()
			)
			.fetch_one(&self.db)
			.await?,
		)
	}

	pub async fn delete(&self, brief: Brief) -> Result<Brief> {
		let id = brief.slug.as_str();
		Ok(
			query_as!(
				Meta,
				r#" DELETE FROM routes WHERE id = ? RETURNING
      id AS "id!: _", kind AS "kind!: _", created AS "created!: _" "#,
				id
			)
			.fetch_one(&self.db)
			.await?
			.map(Brief::from),
		)
	}

	pub async fn purge_expired(&self) -> Result<Vec<Brief>> {
		let retention = self.cfg.mgmt.retention.as_secs().cast_signed();
		query_as!(
			Meta,
			r#"DELETE FROM routes
      WHERE kind != 0
      AND created < CAST(strftime('%s','now') AS INTEGER) - ? RETURNING
      id AS "id!: _", kind AS "kind!: _", created AS "created!: _" "#,
			&retention,
		)
		.fetch_all(&self.db)
		.await?
		.into_iter()
		.map(Brief::from)
		.collect()
	}

  pub async fn insert(&self, data: UploadReady) -> Result<Brief> {
  }
  async fn insert_fixed(&self, data
}
