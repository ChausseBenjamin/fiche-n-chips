use std::error::Error;

use turso::{Connection, Statement};

pub struct Statements {
	pub get_from_path: Statement,
	pub get_from_hash: Statement,
	pub del_from_path: Statement,
	pub del_from_hash: Statement,
	pub insert_paste: Statement,
	pub insert_redir: Statement,
}

impl Statements {
	pub async fn setup(conn: &Connection) -> Result<Self, Box<dyn Error>> {
		Ok(Self {
			get_from_path: conn
				.prepare_cached("SELECT * FROM routes WHERE (id = ?)")
				.await?,
			get_from_hash: conn
				.prepare_cached("SELECT * FROM routes WHERE (hash = ?)")
				.await?,
			del_from_path: conn
				.prepare_cached("DELETE FROM routes WHERE (id = ?)")
				.await?,
			del_from_hash: conn
				.prepare_cached("DELETE FROM routes WHERE (hash = ?)")
				.await?,
			insert_paste: todo!(),
			insert_redir: todo!(),
		})
	}
}
