use actix_web::http::Method;
use actix_web::{HttpRequest, HttpResponse};

use super::super::{Client, RespFmt};
use crate::ui::{self, BOLD, GREEN_FG};
use crate::util::{sorry, IDENT};

pub async fn api_healthcheck(req: &HttpRequest) -> HttpResponse {
	let ansi_resp: bool = match Client::from(req).prefered_format() {
		RespFmt::Ansi => true,
		RespFmt::Plain | RespFmt::Html => false,
	};

	#[allow(clippy::single_match_else)]
	match req.method() {
		&Method::GET => HttpResponse::Ok().body(ui::format!(
			ansi_resp,
			"{}",
			ui::string("ok", &[GREEN_FG, BOLD]),
		)),
		_ => HttpResponse::MethodNotAllowed().insert_header(("Allow", "GET")).body(
			ui::format!(
				ansi_resp,
				"Only `{}` are allowed on the healthcheck endpoint.\n{}",
				ui::string("GET", IDENT),
				sorry()
			),
		),
	}
}
