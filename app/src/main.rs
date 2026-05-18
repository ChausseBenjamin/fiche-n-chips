mod internal;

use std::{fmt::format, io::Result};

use actix_web::{get, web, App, HttpServer, Responder};
use clap::Parser;
use rslog::{attrs, JsonHandler, Logger, SlogLevel};

use crate::internal::{config, server};

#[get("/greet/{id}")]
async fn greet(user_id: web::Path<String>) -> impl Responder {
	format!("<h1>Hey, what's up {user_id}?!!</h1>")
}

#[actix_web::main]
async fn main() -> Result<()> {
	let cfg = config::Config::parse();
	let logger: Logger = cfg.logging.try_into()?;
	rslog::set_default(logger);

	rslog::default().info(
		"starting server",
		attrs!("web-port" => cfg.http.server_port),
	);

	HttpServer::new(|| App::new().configure(server::config))
		.bind(("127.0.0.1", cfg.http.server_port))?
		.workers(2)
		.run()
		.await
}
