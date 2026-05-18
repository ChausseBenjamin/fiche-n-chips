use actix_web::web;

pub mod api;
pub mod basic;
pub mod slug;

pub fn config(cfg: &mut web::ServiceConfig) {
	cfg.configure(basic::config).configure(slug::config);
}
