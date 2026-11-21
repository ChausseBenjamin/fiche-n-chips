use serde::Serialize;

use super::Error;
use super::emitter::AlertMsg;
use crate::alerting::expiry::ContentKind;
use crate::domain::Brief;

#[derive(Serialize)]
pub struct DeletionRequest {
	slug: String,
	kind: ContentKind,
}

impl TryFrom<&Brief> for DeletionRequest {
	type Error = Error;

	fn try_from(brief: &Brief) -> Result<Self, Error> {
		Ok(Self {
			slug: brief.slug.to_string(),
			kind: brief.kind.try_into()?,
		})
	}
}

impl AlertMsg for DeletionRequest {
	const TEMPLATE: &'static str = "fnp-manual";

	fn topic(&self) -> &str { &self.slug }
}
