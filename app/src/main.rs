mod internal;

use std::{fmt::format, io::Result};

use actix_web::{get, web, App, HttpServer, Responder};
use clap::Parser;

use crate::internal::{config, logging};

#[get("/")]
async fn home() -> impl Responder {
	format!("<h1>hello world</h1>")
}

#[get("/greet/{id}")]
async fn greet(user_id: web::Path<String>) -> impl Responder {
	format!("<h1>Hey, what's up {user_id}?!!</h1>")
}

#[actix_web::main]
async fn main() -> Result<()> {
	let cfg = config::Config::parse();
	println!("Starting server on port {}", cfg.http.server_port);

	HttpServer::new(|| App::new().service(home).service(greet))
		.bind(("127.0.0.1", cfg.http.server_port))?
		.workers(2)
		.run()
		.await
}
