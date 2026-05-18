pub mod logging;
use std::{path::PathBuf, time::Duration};

use clap::{Args, Parser};
use logging::LoggingCfg;

#[derive(Parser)]
#[clap(version, about, long_about = None)]
pub struct Config {
	#[clap(flatten)]
	pub server: ServerCfg,
	#[clap(flatten)]
	pub logging: LoggingCfg,
	#[clap(flatten)]
	pub management: ManagementCfg,
	#[clap(flatten)]
	pub pastebin: PastebinCfg,
	#[clap(flatten)]
	pub redirect: RedirectCfg,
}

#[derive(Args)]
#[clap(next_help_heading = "Server")]
pub struct ServerCfg {
	/// Port the HTTP server listens on
	#[arg(long, env = "HTTP_PORT", default_value_t = 9001)] // over 9000...
	pub http_port: u16,
	/// Port the raw TCP pastebin listener binds to
	#[arg(long, env = "PASTE_PORT", default_value_t = 9002)]
	pub paste_port: u16,
	/// Local address to bind to
	#[arg(long, env = "LISTEN_ADDR", default_value_t = Box::from("0.0.0.0"))]
	pub listen_addr: Box<str>,
	/// Address advertised to users in API and TCP responses
	#[arg(long, env = "PUBLIC_ADDR")]
	pub public_addr: Option<Box<str>>,
}

#[derive(Args)]
#[clap(next_help_heading = "Internal")]
pub struct ManagementCfg {
	/// Directory containing store.db
	#[arg(long, env = "DB_PATH", default_value = ".internal/store.db")]
	pub db_path: PathBuf,
	/// How long pastes and redirects are retained
	#[arg(long, env = "RETENTION", default_value = "365days", value_parser = parse_duration)]
	pub retention: Duration,
}

#[derive(Args)]
#[clap(next_help_heading = "Pastebin")]
pub struct PastebinCfg {
	/// Character length of generated paste slugs
	#[arg(long, env = "PASTE_SLUG_LEN", default_value_t = 4)]
	pub slug_length: u8,
	/// Maximum paste size in KB
	#[arg(long, env = "PASTE_MAX_KB", default_value = "256", value_parser = kb_to_bytes::<u32>)]
	pub max_buffer: u32,
}

#[derive(Args)]
#[clap(next_help_heading = "Redirects")]
pub struct RedirectCfg {
	/// Maximum length of a URL submitted for shortening (KB)
	#[arg(long, env = "REDIRECT_MAX_KB", default_value = "2", value_parser = kb_to_bytes::<u32>)]
	pub max_raw: u32,
	/// Maximum character length of generated redirect slugs
	#[arg(long, env = "REDIRECT_SLUG_LEN", default_value_t = 144)]
	pub max_short: u16,
}

pub fn kb_to_bytes<T>(s: &str) -> Result<T, String>
where
	u64: TryInto<T>,
	<u64 as TryInto<T>>::Error: std::fmt::Display,
{
	s.parse::<u64>()
		.map_err(|e| e.to_string())
		.and_then(|kb| kb.checked_mul(1024).ok_or_else(|| "overflows u64".into()))
		.and_then(|b| {
			b.try_into().map_err(|_| {
				format!("{s} KB ({b} bytes) exceeds the maximum for this field")
			})
		})
}

pub fn parse_duration(s: &str) -> Result<Duration, String> {
	humantime::parse_duration(s).map_err(|e| e.to_string())
}
