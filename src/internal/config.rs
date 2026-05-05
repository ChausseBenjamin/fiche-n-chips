use clap::{Args, ValueEnum};

use super::logging::config::{LogFmt, LogOutput};

#[derive(Args)]
#[command(version, about, long_about=none)]
pub struct Config {
	#[command(flatten)]
	http: HttpCfg,
	#[command(flatten)]
	logging: LoggingCfg,
}

#[derive(Args)]
#[command(next_help_heading = "Logging")]
struct HttpCfg {
	// Hosts the website + API
	#[arg(long, env = "HTTP_PORT", default_value_t = 9001)]
	pub http_port: u16, // TODO: use this variable
	// Receives raw TCP for pastebin uploads
	#[arg(long, env = "UPLOAD_PORT", default_value_t = 9002)]
	pub upload_port: u16, // TODO: use this variable
}

#[derive(Args)]
#[command(next_help_heading = "Logging")]
struct LoggingCfg {
	#[arg(long, env = "LOG_FORMAT", default_value_t = LogFmt::Json)]
	pub format: LogFmt,
	#[arg(long, env = "LOG_OUTPUT", default_value_t = LogOutput::Stdout)]
	pub output: LogOutput,
}
