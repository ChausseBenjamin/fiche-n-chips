use clap::{Args, Parser};

use crate::internal::logging::{LogFmt, LogOutput};

#[derive(Parser)]
#[clap(version, about, long_about = None)]
pub struct Config {
	#[clap(flatten)]
	pub http: HttpCfg,
	#[clap(flatten)]
	pub logging: LoggingCfg,
}

#[derive(Args)]
#[clap(next_help_heading = "Logging")]
pub struct LoggingCfg {
	#[arg(long, env = "LOG_FORMAT", default_value_t = LogFmt::Json)]
	pub format: LogFmt,
	#[arg(long, env = "LOG_OUTPUT", default_value_t = LogOutput::Stdout)]
	pub output: LogOutput,
}

#[derive(Args)]
#[clap(next_help_heading = "HTTP")]
pub struct HttpCfg {
	// Hosts the website + API
	#[arg(long, env = "SERVER_PORT", default_value_t = 9001)]
	pub server_port: u16,
	// Receives raw TCP for pastebin uploads
	#[arg(long, env = "UPLOAD_PORT", default_value_t = 9002)]
	pub upload_port: u16,
	// #[arg(long, env = "LISTEN_ADDRESS", default_value_t = "127.0.0.1")]
	// pub listen_addr: &'static str,
}
