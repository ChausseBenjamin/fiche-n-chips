use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use sqlx::sqlite::SqliteAutoVacuum::Incremental;
use sqlx::sqlite::SqliteJournalMode::Wal;
use sqlx::sqlite::SqliteSynchronous::Normal;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::{ConnectOptions, query_as, query_scalar};
use tokio::sync::Mutex;

use crate::config::Config;
use crate::domain::{
	Brief, Checksum, ContentPacked, Novel, Record, RecordStored, RouteKind,
	SHOTGUN_SHELL, Slug, SlugGenerator, UploadPending, UploadReady,
};

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

	pub async fn potential_duplicates(
		&self,
		upload: &UploadPending,
	) -> Result<Vec<RecordStored>> {
		let plain = upload.content();
		let hash = upload.checksum().ok_or(Err(Error::Validation(
			"imposible to validate uniqueness for slug without a hash".to_string(),
		)))?;
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

	pub async fn get_brief(&self, id: &Path) -> Result<Brief> {
		let meta = query_as!(
			Meta,
			r#"SELECT
          id AS "id!: _", kind AS "kind!: _", created AS "created!: _"
          FROM routes WHERE id = ? "#,
			id.to_string_lossy().as_ref(),
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

	pub async fn insert(&self, submission: UploadReady) -> Result<Brief> {
		let UploadReady {
			naming,
			content,
			checksum,
		} = entry;
		let kind: RowKind = RouteKind::from(&content).into();
		let hash = i64::from(checksum.unwrap_or_default());
		let content = content.into_inner(); // Alloc-free move
		match naming {
			Naming::Fixed(slug) => self.insert_fixed(slug, hash, kind, content).await,
			Naming::Generated => self.insert_generated(hash, kind, content).await,
		}
	}

	async fn insert_fixed(
		&self,
		id: Slug,
		hash: i64,
		kind: RowKind,
		content: Vec<u8>,
	) -> Result<Brief> {
		query_as!(Meta, r#"INSERT INTO routes (id, kind, hash, content) VALUES (?, ?, ?, ?)
        RETURNING id AS "id!: _", kind AS "kind!: _", created AS "created!: _" "#,
        slug.as_str(),
        i64::from(kind),
        hash,
        content,
    )
			.fetch_one(&self.db)
			.await?
			.map(Brief::from)
	}

	async fn insert_generated(
		&self,
		hash: i64,
		kind: RowKind,
		content: Vec<u8>,
	) -> Result<Brief> {
		let shotgun = SlugGenerator::generate(&SHOTGUN_SHELL);
		#[rustfmt::skip]
		Ok(query_as!(
			Meta, r#"
        WITH RECURSIVE split(slug, rest, ord) AS (
            SELECT NULL, ? || '/', 0 UNION ALL
            SELECT substr(rest, 1, instr(rest, '/') - 1),
                   substr(rest, instr(rest, '/') + 1),
                   ord + 1
            FROM split WHERE rest <> '' AND instr(rest, '/') > 0 )
        INSERT INTO routes (id, kind, hash, content)
        SELECT slug, ?, ?, ? FROM split
        WHERE ord > 0 AND slug <> ''
          AND NOT EXISTS (SELECT 1 FROM routes AS e WHERE e.id = split.slug)
        ORDER BY ord LIMIT 1
        RETURNING id AS "id!: _", kind AS "kind!: _", created AS "created!: _" "#,
        shotgun,
        i64::from(kind),
        hash,
        content)
            .fetch_optional(&self.db)
            .await?
            .ok_or(Error::Collision(SHOTGUN_ATTEMPTS))?
            .map(Brief::from))
	}
}
