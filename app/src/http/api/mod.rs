use std::path::PathBuf;

use actix_web::{HttpRequest, HttpResponse};

use crate::storage::model::RowEntry;

pub(super) async fn api_post(
	_req: HttpRequest,
	_endpoint: PathBuf,
) -> HttpResponse {
	todo!()
}

pub(super) async fn api_delete(
	_req: HttpRequest,
	_endpoint: RowEntry,
) -> HttpResponse {
	todo!()
}
