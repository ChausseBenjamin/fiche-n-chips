use std::sync::Arc;
use std::time::{Duration, SystemTime};

use actix_web::{HttpRequest, HttpResponse};
use rslog::{attrs, error, warn};

use super::super::Client;
use crate::alerting::{DeletionRequest, MsgEmitter};
use crate::config::Config;
use crate::config::logging::{ERR_KEY, SLUG_KEY};
use crate::http::client::ResponseFmt as RespFmt;
use crate::storage::Storage;
use crate::storage::model::{RowEntry as Row, Storable};
use crate::ui;
use crate::util::{CONTACT, EMPHASIS, IDENT, sorry, success};

pub async fn api_delete(
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
	let max_delay = cfg.management.deletion_window;
	let maintainer_email = cfg.server.maintainer_email.clone();

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
		let entry = match db.queries.del_from_path(id.clone()).await {
			Err(err) => {
				error(
					"failed to delete element from database",
					attrs!(
				  SLUG_KEY => id.clone(),
				  ERR_KEY => err),
				);
				let mut resp = HttpResponse::InternalServerError();
				return resp.body(ui::format!(ansi_resp,
        "An error occured while attempting to delete `{}`.\nIf you still wish to delete this, please contact <{}> about the issue so he may delete it manually.\n{}",
        ui::string(id, IDENT),
        ui::string(maintainer_email, CONTACT),
        sorry(),
      ));
			}
			Ok(v) => v,
		};

		// Manual deletions are quite rare
		// Creating emmitters+Clients here to keeps the code simple
		match (
			MsgEmitter::try_from(&cfg.ntfy),
			DeletionRequest::try_from(&entry),
		) {
			(Ok(emitter), Ok(msg)) => {
				if let Err(e) = emitter.emit(msg).await {
					warn(
						"failed to issue a manual deletion alert",
						attrs!(
										ERR_KEY => e,
										SLUG_KEY => id.clone(),
						),
					)
				}
			}
			(Err(_), _) => (
        // alerting::Error::Unset is ok, it's a runtime configuration
      ),
			(_, Err(e)) => error(
				"failed to parse slug type to issue an alert",
				attrs!(
								ERR_KEY => e,
								SLUG_KEY => id.clone(),
				),
			),
		}

		let mut resp = HttpResponse::Ok();
		return resp.body(ui::format!(
			ansi_resp,
			"Successfully deleted `{}`, {}\n",
			ui::string(id, IDENT),
			success(),
		));
	}

	let mut resp = HttpResponse::Forbidden();
	resp.body(ui::format!(ansi_resp,
    "You are trying to delete `{}` but it has been created more than {} ago.\n It can no longer be deleted automatically (created {} ago).\nPlease contact <{}> if you still wish to have this content removed.\n{}",
    ui::string(id, IDENT),
    humantime::format_duration(max_delay),
    ui::string(humantime::format_duration(elapsed), EMPHASIS),
    ui::string(maintainer_email, CONTACT),
    sorry(),
  ))
}
