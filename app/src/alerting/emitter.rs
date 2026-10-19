use awc::Client;
use serde::Serialize;

use super::Error;
use crate::config::NtfyCfg;
use crate::util::Secret;

#[derive(Clone, Copy, Serialize)]
#[serde(into = "u8")]
#[repr(u8)]
pub enum MessagePriority {
	Max    = 5,
	High   = 4,
	Normal = 3,
	Low    = 2,
	Min    = 1,
}

impl From<MessagePriority> for u8 {
	fn from(pri: MessagePriority) -> Self { pri as u8 }
}

pub trait AlertMsg: Serialize {
	/// Filename the ntfy server has (without the `.yml` extension)
	const TEMPLATE: &'static str;
	fn topic(&self) -> &str;
	fn template(&self) -> &str { Self::TEMPLATE }
}

pub struct MsgEmitter {
	client:        Client,
	topic_prefix:  String,
	ntfy_endpoint: String,
	ntfy_token:    Option<Secret<String>>,
}

impl MsgEmitter {
	pub async fn emit(
		&self,
		msg: impl AlertMsg + Serialize,
	) -> Result<(), Error> {
		let mut req = self
			.client
			.post(format!(
				"{}/{}{}",
				self.ntfy_endpoint,
				self.topic_prefix,
				msg.topic()
			))
			.content_type("application/json")
			.insert_header(("Template", msg.template()));
		if let Some(token) = &self.ntfy_token {
			req = req.bearer_auth(token.expose());
		}
		let resp = req.send_json(&msg).await?;
		if !resp.status().is_success() {
			return Err(Error::from_response(resp).await);
		}
		Ok(())
	}
}

impl TryFrom<&NtfyCfg> for MsgEmitter {
	type Error = Error;

	fn try_from(cfg: &NtfyCfg) -> Result<Self, Self::Error> {
		let ntfy_endpoint = match cfg.ntfy_url.as_ref() {
			Some(url) => url.as_ref().trim_end_matches('/').to_string(),
			None => return Err(Error::Unset),
		};
		let client = Client::default();
		let topic_prefix = cfg.prefix.as_deref().unwrap_or_default().to_string();
		let ntfy_token = cfg.api_key.clone();
		Ok(Self {
			client,
			topic_prefix,
			ntfy_endpoint,
			ntfy_token,
		})
	}
}
