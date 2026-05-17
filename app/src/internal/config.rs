use clap::{Args, Parser};
use rslog::{SlogLevel, INFO};

use crate::internal::logging::{LogFmt, LogOutput, LoggingCfg};

#[derive(Parser)]
#[clap(version, about, long_about = None)]
pub struct Config {
	#[clap(flatten)]
	pub http: HttpCfg,
	#[clap(flatten)]
	pub logging: LoggingCfg,
}

#[derive(Args)]
#[clap(next_help_heading = "HTTP")]
pub struct HttpCfg {
	/// Where the main website and urls are served
	#[arg(long, env = "SERVER_PORT", default_value_t = 9001)]
	pub server_port: u16,
	/// Where raw tcp data is received for pastebins
	#[arg(long, env = "UPLOAD_PORT", default_value_t = 9002)]
	pub upload_port: u16,
	/// Network address used to listen for incoming connections
	#[arg(long, env = "LISTEN_ADDRESS", default_value_t = Box::from("0.0.0.0"))]
	pub listen_addr: Box<str>,
}
