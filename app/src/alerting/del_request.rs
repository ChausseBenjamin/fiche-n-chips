use serde::Serialize;

use super::Error;
use super::emitter::AlertMsg;
use crate::alerting::expiry::ContentKind;
use crate::storage::model::DeletedRoute;

#[derive(Serialize)]
pub struct DeletionRequest {
	slug: String,
	kind: ContentKind,
}

impl TryFrom<&DeletedRoute> for DeletionRequest {
	type Error = Error;

	fn try_from(route: &DeletedRoute) -> Result<Self, Error> {
		Ok(Self {
			slug: route.id.clone(),
			kind: route.kind.try_into()?,
		})
	}
}

impl AlertMsg for DeletionRequest {
	const TEMPLATE: &'static str = "fnp-manual";

	fn topic(&self) -> &str { &self.slug }
}
