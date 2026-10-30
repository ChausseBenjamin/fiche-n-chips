use std::sync::Arc;
use std::time::{Duration, SystemTime};

use actix_web::{HttpRequest, HttpResponse};
use rslog::{attrs, error, warn};

use super::super::Client;
use crate::alerting::{DeletionRequest, Error as AlertingError, MsgEmitter};
use crate::config::Config;
use crate::config::logging::{ERR_KEY, SLUG_KEY};
use crate::domain::{Brief, RouteKind};
use crate::http::client::ResponseFmt as RespFmt;
use crate::storage::{Error as StorageError, Storage};
use crate::ui;
use crate::util::{CONTACT, EMPHASIS, IDENT, sorry, success};

/// Everything that can prevent a deletion, plus the slug it applies to.
struct DeleteError {
	slug:  String,
	fault: DeleteFault,
}

enum DeleteFault {
	Reserved,
	TooOld {
		max:     Duration,
		elapsed: Duration,
	},
	Storage(StorageError),
}

impl DeleteError {
	fn respond(self, ansi_resp: bool, maintainer_email: &str) -> HttpResponse {
		let Self { slug, fault } = self;
		match fault {
			DeleteFault::Reserved => {
				HttpResponse::Forbidden().body(ui::format!(
					ansi_resp,
					"Deleting statically served files such as `{}` isn't permitted.\n{}",
					slug,
					sorry(),
				))
			}
			DeleteFault::TooOld { max, elapsed } => {
				HttpResponse::Forbidden().body(ui::format!(ansi_resp,
          "You are trying to delete `{}` but it has been created more than {} ago.\n\
          It can no longer be deleted automatically (created {} ago).\n\
          Please contact <{}> if you still wish to have this content removed.\n{}",
          ui::string(slug, IDENT),
          humantime::format_duration(max),
          ui::string(humantime::format_duration(elapsed), EMPHASIS),
          ui::string(maintainer_email, CONTACT),
          sorry(),
        ))
			}
			DeleteFault::Storage(e) => {
				error(
					"failed to delete element from database",
					attrs!(SLUG_KEY => slug.clone(), ERR_KEY => e),
				);
				HttpResponse::InternalServerError().body(ui::format!(ansi_resp,
          "An error occured while attempting to delete `{}`.\n\
          If you still wish to delete this, please contact <{}> about the issue \
          so he may delete it manually.\n{}",
          ui::string(slug, IDENT),
          ui::string(maintainer_email, CONTACT),
          sorry(),
        ))
			}
		}
	}
}

pub async fn api_delete(
	req: &HttpRequest,
	endpoint: Brief,
	cfg: &Config,
	db: Arc<Storage>,
) -> HttpResponse {
	let ansi_resp: bool = match Client::from(req).prefered_format() {
		RespFmt::Ansi => true,
		RespFmt::Plain | RespFmt::Html => false,
	};

	match handle_delete(endpoint, cfg, &db, ansi_resp).await {
		Ok(resp) => resp,
		Err(e) => e.respond(ansi_resp, &cfg.server.maintainer_email),
	}
}

async fn handle_delete(
	endpoint: Brief,
	cfg: &Config,
	db: &Storage,
	ansi_resp: bool,
) -> Result<HttpResponse, DeleteError> {
	let slug = endpoint.slug.to_string();

	if let RouteKind::Reserved = endpoint.kind {
		return Err(DeleteError {
			slug,
			fault: DeleteFault::Reserved,
		});
	}

	let elapsed = SystemTime::now()
		.duration_since(endpoint.created)
		.unwrap_or(Duration::ZERO);
	let max = cfg.mgmt.deletion_window;
	if elapsed > max {
		return Err(DeleteError {
			slug,
			fault: DeleteFault::TooOld { max, elapsed },
		});
	}

	let brief = db.delete(endpoint).await.map_err(|e| DeleteError {
		slug,
		fault: DeleteFault::Storage(e),
	})?;

	issue_alert(&brief, cfg).await;

	Ok(HttpResponse::Ok().body(ui::format!(
		ansi_resp,
		"Successfully deleted `{}`, {}\n",
		ui::string(brief.slug, IDENT),
		success(),
	)))
}

/// Deletion alerts are best-effort: a failure here never changes the response.
async fn issue_alert(brief: &Brief, cfg: &Config) {
	let slug = brief.slug.to_string();
	match (
		MsgEmitter::try_from(&cfg.ntfy),
		DeletionRequest::try_from(brief),
	) {
		(Ok(emitter), Ok(msg)) => {
			if let Err(e) = emitter.emit(msg).await {
				warn(
					"failed to issue a manual deletion alert",
					attrs!(ERR_KEY => e, SLUG_KEY => slug),
				);
			}
		}
		(Err(AlertingError::Unset), _) => (
			// alerting::Error::Unset is ok, it's a runtime configuration
		),
		(Err(e), _) => error(
			"failed to set up the alert emitter",
			attrs!(ERR_KEY => e, SLUG_KEY => slug),
		),
		(_, Err(e)) => error(
			"failed to parse slug type to issue an alert",
			attrs!(ERR_KEY => e, SLUG_KEY => slug),
		),
	}
}
