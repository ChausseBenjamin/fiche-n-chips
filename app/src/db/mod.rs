use turso::Builder;

pub struct Storage {
	database: turso::Database,
}

impl Storage {
	pub async fn new() {
		let db = Builder::new_local("test.db").build().await;
		let conn = db.expect("successful DB creation required").connect();
	}
}
