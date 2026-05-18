use actix_web::{get, web, Responder};

pub(super) fn config(cfg: &mut web::ServiceConfig) {
	cfg.service(home);
}

#[get("/")]
async fn home() -> impl Responder {
	format!("<h1>hello world</h1>")
}
