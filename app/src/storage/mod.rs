use std::path::Path;
use std::time::Duration;

use sqlx::sqlite::SqliteAutoVacuum::Incremental;
use sqlx::sqlite::SqliteJournalMode::Wal;
use sqlx::sqlite::SqliteSynchronous::Normal;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::{query_as, query_file_as, query_file_scalar, query_scalar};
use tokio::sync::Mutex;

use crate::config::Config;
use crate::domain::{
	Brief, Entry, EntryPacked, Naming, RecordPacked, RouteKind, SHOTGUN_ATTEMPTS,
	SHOTGUN_SHELL, Slug, SlugGenerator, UploadPending, UploadReady,
};
use crate::storage::primitives::Payload;

mod error;
mod primitives;

pub use error::{Error, Result};
use primitives::{Meta, Route, RowKind};

pub struct StorageCfg {
	retention: Duration,
}

pub struct Storage {
	db:     SqlitePool,
	cfg:    StorageCfg,
	shrink: Mutex<()>,
}

impl Storage {
	pub async fn setup(cfg: &Config) -> Result<Self> {
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
		let db = SqlitePoolOptions::new()
			.max_connections(10)
			.acquire_timeout(Duration::from_secs(5))
			.connect_with(opts)
			.await?;
		sqlx::migrate!("src/storage/migrations").run(&db).await?;
		Ok(Self {
			cfg: StorageCfg {
				retention: cfg.mgmt.retention,
			},
			db,
			shrink: Mutex::new(()),
		})
	}

	pub async fn delete(&self, brief: Brief) -> Result<Brief> {
		let id = brief.slug.as_str();
		let meta = query_file_as!(Meta, "src/storage/statements/delete.sql", id)
			.fetch_one(&self.db)
			.await?;
		Ok(Brief::from(meta))
	}

	pub async fn exists(&self, id: &Slug) -> Result<bool> {
		Ok(
			query_file_scalar!("src/storage/statements/exists.sql", id.as_str())
				.fetch_one(&self.db)
				.await?,
		)
	}

	pub async fn get(&self, id: &Path) -> Result<Option<EntryPacked>> {
		let payload = query_file_as!(
			Payload,
			"src/storage/statements/get.sql",
			id.to_string_lossy()
		)
		.fetch_optional(&self.db)
		.await?;
		payload.map(EntryPacked::try_from).transpose()
	}

	pub async fn get_brief(&self, id: &Path) -> Result<Brief> {
		let meta = query_file_as!(
			Meta,
			"src/storage/statements/get_brief.sql",
			id.to_string_lossy(),
		)
		.fetch_one(&self.db)
		.await?;
		Ok(meta.into())
	}

	pub async fn insert(&self, submission: UploadReady) -> Result<Brief> {
		let UploadReady {
			naming,
			content,
			checksum,
		} = submission;
		let kind: RowKind = RouteKind::from(&content).into();
		let hash = checksum.map(i64::from);
		let content = content.into_inner(); // Alloc-free move
		match naming {
			Naming::Fixed(slug) => self.insert_fixed(slug, hash, kind, content).await,
			Naming::Generated => self.insert_generated(hash, kind, content).await,
		}
	}

	async fn insert_fixed(
		&self,
		id: Slug,
		hash: Option<i64>,
		kind: RowKind,
		content: Box<[u8]>,
	) -> Result<Brief> {
		let meta = query_file_as!(
			Meta,
			"src/storage/statements/insert_fixed.sql",
			id.as_str(),
			i64::from(kind),
			hash,
			content,
		)
		.fetch_optional(&self.db)
		.await?
		// `ON CONFLICT ... DO NOTHING`: only 1 insert can
		// win if two concurrent requests want the same slug.
		// NOTHING => detect if we lost the race (we get None instead of a brief)
		.ok_or(Error::SlugTaken(id))?;
		Ok(Brief::from(meta))
	}

	#[rustfmt::skip]
	async fn insert_generated(
		&self,
		hash: Option<i64>,
		kind: RowKind,
		content: Box<[u8]>,
	) -> Result<Brief> {
		let shotgun = SlugGenerator::generate(&SHOTGUN_SHELL);
		let meta = query_file_as!(
			Meta, "src/storage/statements/insert_generated.sql",
      shotgun,
        i64::from(kind),
        hash,
        content)
            .fetch_optional(&self.db)
            .await?
            .ok_or(Error::Exhausted(SHOTGUN_ATTEMPTS))?;
		Ok(Brief::from(meta))
	}

	pub async fn potential_duplicates(
		&self,
		upload: &UploadPending,
	) -> Result<Vec<RecordPacked>> {
		let Some(hash) = upload.checksum() else {
			// hashes only exist for pastes; redirects are never deduplicated.
			return Ok(Vec::new());
		};
		query_file_as!(
			Route,
			"src/storage/statements/potential_duplicates.sql",
			i64::from(hash),
		)
		.fetch_all(&self.db)
		.await?
		.into_iter()
		.map(RecordPacked::try_from)
		.collect()
	}

	pub async fn purge_expired(&self) -> Result<Vec<Brief>> {
		let retention = self.cfg.retention.as_secs().cast_signed();
		let expired = query_file_as!(
			Meta,
			"src/storage/statements/purge_expired.sql",
			&retention,
		)
		.fetch_all(&self.db)
		.await?;
		Ok(expired.into_iter().map(Brief::from).collect())
	}
}
