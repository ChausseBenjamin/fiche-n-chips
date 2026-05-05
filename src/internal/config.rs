use clap::{Args, ValueEnum};

use super::logging::config::{LogFmt, LogOutput};

#[derive(Parser)]
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
	#[arg(long, env = "SERVER_PORT", default_value_t = 9001)]
	pub server_port: u16,
	// Receives raw TCP for pastebin uploads
	#[arg(long, env = "UPLOAD_PORT", default_value_t = 9002)]
	pub upload_port: u16,
	// #[arg(long, env = "LISTEN_ADDRESS", default_value_t = "127.0.0.1")]
	// pub listen_addr: &'static str,
}

#[derive(Args)]
#[command(next_help_heading = "Logging")]
struct LoggingCfg {
	#[arg(long, env = "LOG_FORMAT", default_value_t = LogFmt::Json)]
	pub format: LogFmt,
	#[arg(long, env = "LOG_OUTPUT", default_value_t = LogOutput::Stdout)]
	pub output: LogOutput,
}
