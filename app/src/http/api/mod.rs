use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use actix_web::{HttpRequest, HttpResponse};
use rslog::{attrs, error};

use crate::config::logging::ERR_KEY;
use crate::config::Config;
use crate::http::client::{Client, ResponseFmt as RespFmt};
use crate::storage::model::{RowEntry as Row, Storable};
use crate::storage::Storage;
use crate::ui;
use crate::util::sorry;

pub(super) async fn api_post(
	_req: HttpRequest,
	_endpoint: PathBuf,
) -> HttpResponse {
	todo!()
}

pub(super) async fn api_delete(
	req: &HttpRequest,
	endpoint: Row,
	cfg: &Config,
	db: Arc<Storage>,
) -> HttpResponse {
	let ansi_resp: bool = match Client::from(req).prefered_format() {
		RespFmt::Ansi => true,
		RespFmt::Plain | RespFmt::Html => false,
	};
	let now = SystemTime::now();
	let max_delay = cfg.management.retention;

	let (id, created): (String, SystemTime) = match endpoint {
		Row::Reserved(res) => {
			let mut resp = HttpResponse::Forbidden();
			return resp.body(ui::format!(
				ansi_resp,
				"Deleting statically served files such as `{}` isn't permitted.\n{}",
				res.id(),
				sorry(),
			));
		}
		Row::Redirect(redir, v) => (redir.origin().to_string(), v),
		Row::Paste(paste, v) => (paste.slug(), v),
	};

	let elapsed = now.duration_since(created).unwrap_or(Duration::ZERO);
	if elapsed < max_delay {
		db.queries.del_from_path(id).await.or_else(|err| {
      error("failed to delete element from database", attrs!(
              "path" => id,
              ERR_KEY => err,
      ));
      let mut resp = HttpResponse::InternalServerError();
      return resp.body(ui::format!(ansi_resp,
        "An error occured while attempting to delete `{}`.\nIf you still wish to delete this, please contact <{}> about the issue so he may delete it manually.\n{}",
        ui::string(id, &[ui::BOLD, ui::YELLOW_FG]),
        ui::string(cfg.server.maintainer_email, &[ui::BOLD, ui::UNDERLINE]),
        sorry(),
      ));
		});
	};
	let mut resp = HttpResponse::Forbidden();
	return resp.body(ui::format!(ansi_resp,
    "You are trying to delete `{}` but it has been created more than {} ago.\n It can no longer be deleted automatically (created {} ago).\nPlease contact <{}> if you still wish to have this content removed.\n{}",
    ui::string(id, &[ui::BOLD, ui::YELLOW_FG]),
    humantime::format_duration(max_delay),
    ui::string(humantime::format_duration(elapsed), &[ui::BLUE_FG]),
    ui::string(cfg.server.maintainer_email, &[ui::BOLD, ui::UNDERLINE]),
    sorry(),
  ));
}
