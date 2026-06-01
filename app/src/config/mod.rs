pub mod logging;
use std::path::PathBuf;
use std::time::Duration;

use clap::{Args, Parser};
use logging::LoggingCfg;

use crate::util::StorageSize;

#[derive(Parser)]
#[command(version, about="A fiche-like pastebin with extra snacks", long_about = None)]
pub struct Config {
	/// Confirations related to networking and communication with the outside
	/// world
	#[clap(flatten)]
	pub server:     ServerCfg,
	/// Settings exclusive to the management of pastebins
	/// (creation/deletion/etc...)
	#[clap(flatten)]
	pub pastebin:   PastebinCfg,
	/// Settings exclusive to the management of redirect urls
	/// (creation/deletion/etc...)
	#[clap(flatten)]
	pub redirect:   RedirectCfg,
	/// Configurations that are either internal or relate to multiple areas at
	/// once (ex: redirect+pastebin)
	#[clap(flatten)]
	pub management: ManagementCfg,
	/// Formatting and miscellaneous informations related to runtime logs
	#[clap(flatten)]
	pub logging:    LoggingCfg,
}

#[derive(Args)]
#[clap(next_help_heading = "Server")]
pub struct ServerCfg {
	/// Address advertised to users in API and TCP responses
	#[arg(long, env = "PUBLIC_ADDR", default_value = "http://localhost:9001")]
	pub public_addr:      Box<str>,
	/// Email of the person maintaining the server
	#[arg(long, env = "MAINTAINER_EMAIL", default_value = "john@example.com")]
	pub maintainer_email: Box<str>,
	/// Address to bind to (use 0.0.0.0 to be accessible from other devices)
	#[arg(long, env = "LISTEN_ADDR", default_value_t = Box::from("127.0.0.1"))]
	pub listen_addr:      Box<str>,
	/// Port the HTTP server listens on
	#[arg(long, env = "HTTP_PORT", default_value_t = 9001)] // over 9000...
	pub http_port: u16,
	/// Port the raw TCP pastebin listener binds to
	#[arg(long, env = "PASTE_PORT", default_value_t = 9002)]
	pub paste_port:       u16,
	/// Directory containing hard-coded html files the server should host
	/// (files packaged in the binary will be used if none is provided)
	#[arg(long, env = "HTML_DIR")]
	pub html_dir:         Option<PathBuf>,
}

#[derive(Args)]
#[clap(next_help_heading = "Internal")]
pub struct ManagementCfg {
	/// Directory containing store.db
	#[arg(long, env = "DB_PATH", default_value = ".internal/store.db")]
	pub db_path:           PathBuf,
	/// How much of the database to cache in memory
	#[arg(long, env = "DB_CACHE", default_value = "32MB")]
	pub db_cache:          StorageSize,
	/// Maximum size the database is allowed to have
	#[arg(long, env = "DB_MAX_SIZE", default_value = "10GB")]
	pub db_size:           StorageSize,
	/// Seed location used to anonymize user IPs
	#[arg(long, env = "SECRET_SEED_PATH", default_value = ".internal/seed")]
	pub seed_path:         PathBuf,
	/// Maximum number of uploads (short-urls or pastes) a user can make in a set
	/// period
	#[arg(long, env = "USER_RATE_LIMIT", default_value_t = 100)]
	pub rate_limit:        u16,
	/// Time window for the rate limit (e.g. "1h", "15m", "1d")
	#[arg(long, env = "USER_RATE_LIMIT_PERIOD", default_value = "1h", value_parser = parse_duration)]
	pub rate_limit_period: Duration,
	/// Period after creation during which a user is allowed to delete an entry
	#[arg(long, env = "USER_DELETION_WINDOW", default_value = "10m", value_parser = parse_duration)]
	pub deletion_window:   Duration,
	/// How long pastes and redirects are retained
	#[arg(long, env = "RETENTION", default_value = "365days", value_parser = parse_duration)]
	pub retention:         Duration,
	/// Character length of generated slugs
	/// - always the case for paste
	/// - used for shortening when no short url is provided
	#[arg(long, env = "SLUG_LEN", default_value_t = 4)]
	pub slug_length:       u8,
	/// At which time interval the server should check for expired entries and
	/// clean them up
	#[arg(long, env = "CLEANUP_INTERVAL", default_value = "1h", value_parser = parse_duration)]
	pub cleanup_interval:  Duration,
	/// Graceful shutdown timeout before forced termination
	#[arg(long, env = "SHUTDOWN_TIMEOUT", default_value = "10s", value_parser = parse_duration)]
	pub shutdown_timeout:  Duration,
	/// A planned feature is to issue notifications using ntfy.sh about
	/// various events. Ex:
	/// - warning 1month before expiry
	/// - redirect has expired
	/// - paste/redirect was prematurely deleted (ex: lack of storage)
	///
	/// Setting this to `https://ntfy.sh` (or any hosted instance)
	/// will enable these notification subscription endpoints once implemented
	#[arg(long, env = "NTFY_SERVER")]
	ntfy_server:           Option<Box<str>>,
	/// url path prefix used for every notification subscription path when
	/// issuing calls to a ntfy instance
	#[arg(long, env = "NTFY_PREFIX", default_value = "fiche-n-chips-")]
	ntfy_prefix:           Option<Box<str>>,
}

#[derive(Args)]
#[clap(next_help_heading = "Pastebin")]
pub struct PastebinCfg {
	/// Maximum paste size
	#[arg(long, env = "PASTE_MAX_SIZE", default_value = "256KB")]
	pub max_buffer:   StorageSize,
	/// Per-read timeout for paste uploads
	#[arg(long, env = "PASTE_READ_TIMEOUT", default_value = "1s", value_parser = parse_duration)]
	pub read_timeout: Duration,
}

#[derive(Args)]
#[clap(next_help_heading = "Redirects")]
pub struct RedirectCfg {
	/// Maximum length of a URL submitted for shortening
	#[arg(long, env = "REDIRECT_MAX_SIZE", default_value = "2KB")]
	pub max_raw:   StorageSize,
	/// Maximum character length of generated redirect slugs
	#[arg(long, env = "REDIRECT_SLUG_LIMIT", default_value_t = 144)]
	pub max_short: u16,
}

pub fn parse_duration(s: &str) -> Result<Duration, String> {
	humantime::parse_duration(s).map_err(|e| e.to_string())
}
