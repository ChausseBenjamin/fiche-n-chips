use std::time::{Duration, SystemTime};

use const_format::formatcp;
use turso::{Connection, Statement, Value};

use super::error::Error;
use super::model::{
	DeletedRoute, ExpiringRoute, PasteRecord, Reserved, RouteKind, RouteRow,
	RowEntry, Storable,
};
use crate::util::{from_unix_timestamp, unix_secs_ceil, unix_secs_floor};

const DEL_MISSING_RESERVED: &str = formatcp!(
	"DELETE FROM routes WHERE kind = {} AND ID NOT IN",
	RouteKind::Reserved as u8
);

const INSERT_ROUTES_SAFE: &str =
	"INSERT OR IGNORE INTO routes (id, kind, hash, content) VALUES";

#[derive(Clone, Copy)]
pub struct DbSize {
	page_count: u64,
	freelist:   u64,
	page_size:  u64,
}

impl DbSize {
	pub fn physical(self) -> u64 {
		self.page_count.saturating_mul(self.page_size)
	}

	pub fn live(self) -> u64 {
		self
			.page_count
			.saturating_sub(self.freelist)
			.saturating_mul(self.page_size)
	}
}

pub struct Statements {
	conn:                Connection,
	del_expired:         Statement,
	del_from_path:       Statement,
	get_created_between: Statement,
	get_from_path:       Statement,
	get_slugs_by_hash:   Statement,
	insert_route:        Statement,
	shrink_batch:        Statement,
	slug_exists:         Statement,
}

impl Statements {
	pub async fn setup(conn: &Connection) -> Result<Self, Error> {
		Ok(Self {
			conn:                conn.clone(),
			del_expired:         conn
				.prepare_cached(include_str!("sql/del_expired.sql"))
				.await?,
			del_from_path:       conn
				.prepare_cached(include_str!("sql/del_from_path.sql"))
				.await?,
			get_created_between: conn
				.prepare_cached(include_str!("sql/get_created_between.sql"))
				.await?,
			get_from_path:       conn
				.prepare_cached(include_str!("sql/get_from_path.sql"))
				.await?,
			get_slugs_by_hash:   conn
				.prepare_cached(include_str!("sql/get_slugs_by_hash.sql"))
				.await?,
			insert_route:        conn
				.prepare_cached(include_str!("sql/insert_route.sql"))
				.await?,
			shrink_batch:        conn
				.prepare_cached(include_str!("sql/shrink_batch.sql"))
				.await?,
			slug_exists:         conn
				.prepare_cached(include_str!("sql/slug_exists.sql"))
				.await?,
		})
	}

	pub async fn get_from_path(&self, path: String) -> Result<RowEntry, Error> {
		let mut cmd = self.get_from_path.clone();
		let row = cmd.query_row([path]).await?;
		RouteRow {
			id:      row.get(0)?,
			kind:    (row.get::<i64>(1)?).try_into()?,
			hash:    row.get(2)?,
			content: row.get(3)?,
			created: Some(from_unix_timestamp(row.get::<i64>(4)?)),
		}
		.try_into()
	}

	pub async fn get_slugs_by_hash(
		&self,
		hash: u64,
	) -> Result<Vec<PasteRecord>, Error> {
		let mut cmd = self.get_slugs_by_hash.clone();
		let mut rows = cmd.query([hash.cast_signed()]).await?;
		let mut results: Vec<PasteRecord> = Vec::new();
		while let Some(row) = rows.next().await? {
			results.push(PasteRecord {
				slug:    row.get(0)?,
				content: (row.get::<Option<Vec<u8>>>(1)?).unwrap_or_default(),
			});
		}
		Ok(results)
	}

	pub async fn get_created_between(
		&self,
		from: SystemTime,
		until: SystemTime,
	) -> Result<Vec<ExpiringRoute>, Error> {
		let mut cmd = self.get_created_between.clone();
		let mut rows =
			cmd.query([unix_secs_ceil(from), unix_secs_floor(until)]).await?;
		let mut results: Vec<ExpiringRoute> = Vec::new();
		while let Some(row) = rows.next().await? {
			results.push(ExpiringRoute {
				id:      row.get(0)?,
				kind:    (row.get::<i64>(1)?).try_into()?,
				created: from_unix_timestamp(row.get::<i64>(2)?),
			});
		}
		Ok(results)
	}

	pub async fn slug_exists(&self, path: &str) -> Result<bool, Error> {
		let mut cmd = self.slug_exists.clone();
		let row = cmd.query_row([path]).await?;
		let exists = row.get::<i64>(0)?;
		Ok(exists == 1)
	}

	pub async fn del_expired(
		&self,
		range: Duration,
	) -> Result<Vec<ExpiringRoute>, Error> {
		let mut cmd = self.del_expired.clone();
		let mut rows = cmd.query([range.as_secs().cast_signed()]).await?;
		let mut results: Vec<ExpiringRoute> = Vec::new();
		while let Some(row) = rows.next().await? {
			results.push(ExpiringRoute {
				id:      row.get(0)?,
				kind:    (row.get::<i64>(1)?).try_into()?,
				created: from_unix_timestamp(row.get::<i64>(2)?),
			});
		}
		Ok(results)
	}

	pub async fn del_from_path(
		&self,
		path: String,
	) -> Result<DeletedRoute, Error> {
		let mut cmd = self.del_from_path.clone();
		let row = cmd.query_row([path]).await?;
		Ok(ExpiringRoute {
			id:      row.get(0)?,
			kind:    (row.get::<i64>(1)?).try_into()?,
			created: from_unix_timestamp(row.get::<i64>(2)?),
		})
	}

	pub async fn delete_oldest_batch(&self) -> Result<Vec<ExpiringRoute>, Error> {
		let mut cmd = self.shrink_batch.clone();
		let mut rows = cmd.query(()).await?;
		let mut results = Vec::new();
		while let Some(row) = rows.next().await? {
			results.push(ExpiringRoute {
				id:      row.get(0)?,
				kind:    (row.get::<i64>(1)?).try_into()?,
				created: from_unix_timestamp(row.get::<i64>(2)?),
			});
		}
		Ok(results)
	}

	/// `PRAGMA`s are not re-evaluated when a prepared statement is stepped
	/// again in Turso 0.6, so query them fresh on every call.
	pub async fn db_size(&self) -> Result<DbSize, Error> {
		Ok(DbSize {
			page_count: self.pragma_u64("page_count").await?,
			freelist:   self.pragma_u64("freelist_count").await?,
			page_size:  self.pragma_u64("page_size").await?,
		})
	}

	pub async fn vacuum(&self) -> Result<(), Error> {
		self.conn.execute("VACUUM", ()).await?;
		Ok(())
	}

	async fn pragma_u64(&self, name: &str) -> Result<u64, Error> {
		let sql = format!("PRAGMA {name}");
		let mut rows = self.conn.query(&sql, ()).await?;
		let row = rows
			.next()
			.await?
			.ok_or_else(|| Error::Setup(format!("failed to query {name}")))?;
		let value: i64 = row.get(0)?;
		u64::try_from(value)
			.map_err(|e| Error::Setup(format!("invalid {name} value: {e}")))
	}

	pub async fn sync_reserved(&self, paths: &[Reserved]) -> Result<u64, Error> {
		let mut conn = self.conn.clone();
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

	pub async fn insert(
		&self,
		route: &impl Storable,
	) -> Result<SystemTime, Error> {
		let id = Value::from(route.id().to_string());
		let kind = Value::from(route.kind() as u8);
		let hash = Value::from(route.hash());
		let content = Value::from(route.content().map(<[u8]>::to_vec));

		let mut cmd = self.insert_route.clone();
		let row = cmd.query_row([id, kind, hash, content]).await?;
		let created_unix: i64 = row.get(0)?;
		Ok(from_unix_timestamp(created_unix))
	}
}
