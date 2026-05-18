use actix_web::{get, web, Responder};

pub(super) fn config(cfg: &mut web::ServiceConfig) {
	cfg.service(greet);
}

#[get("/{id}")]
async fn greet(user_id: web::Path<String>) -> impl Responder {
	format!("<h1>Hey, what's up {user_id}?!!</h1>")
}
