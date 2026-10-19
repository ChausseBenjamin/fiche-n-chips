use std::time::{Duration, SystemTime};

use turso::{Connection, Statement, Value};

use super::error::Error;
use super::model::{
	DeletedRoute, ExpiringRoute, PasteRecord, RouteRow, RowEntry, Storable,
};
use crate::util::{from_unix_timestamp, unix_secs_ceil, unix_secs_floor};

pub struct Statements {
	del_expired:         Statement,
	del_from_path:       Statement,
	get_created_between: Statement,
	get_from_path:       Statement,
	get_slugs_by_hash:   Statement,
	insert_route:        Statement,
	slug_exists:         Statement,
}

impl Statements {
	pub async fn setup(conn: &Connection) -> Result<Self, Error> {
		Ok(Self {
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
