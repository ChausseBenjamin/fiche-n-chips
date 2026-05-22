
#[derive(Clone)]
pub struct Statements({
	get_from_path:     Statement,
	get_slugs_by_hash: Statement,
	slug_exists:       Statement,
	del_expired:       Statement,
	del_from_path:     Statement,
	insert_route:      Statement,
});

impl Statements {
  /// statements are shadowed+private in an inner struct
	pub async fn setup(conn: &Connection) -> Result<Self, Error> {
		Ok(Self({
			get_from_path: conn
				.prepare_cached("SELECT (id, kind, hash, content, created_on) FROM routes WHERE id = ?")
				.await?,
			get_slugs_by_hash: conn
				.prepare_cached("SELECT (id, content) FROM routes WHERE hash = ?")
				.await?,
      slug_exists: panic!("...")
    }))
  }
  // ...
}
  /// Public API can have the same name as the prepared statements
  /// but returns values which are type-safe and pre-mapped to structs
	pub async fn get_from_path(&mut self, path: String) -> Result<Route, Error> {
		let row = self.0.get_from_path.query_row([path]).await?;
		Ok(
			AgnosticRoute {
				id: row.get(0)?,
				kind: (row.get::<i64>(1)?).try_into()?,
				hash: row.get(2)?,
				content: row.get(3)?,
				created: from_unix_timestamp(row.get::<i64>(4)?),
			}
			.into(),
		)
	}
  // ...
	pub async fn slug_exists(&mut self, path: String) -> Result<bool, Error> {
		let row = self.0.slug_exists.query_row([path]).await?;
		let exists: u64 = row.get::<i64>(0)? as u64;
		exists == 1
	}
  // ...
}
