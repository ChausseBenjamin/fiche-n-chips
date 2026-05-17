use core::fmt;
use std::{fmt::Display, io, path::PathBuf, str::FromStr};

use clap::{Args, ValueEnum};
use rslog::{SlogLevel, DEBUG, ERROR, INFO, WARN};

use crate::internal::util;

const DEFAULT_LOGFILE: &str = "/var/log/fiche-n-chips.log";

#[derive(Args)]
#[clap(next_help_heading = "Logging")]
pub struct LoggingCfg {
	#[arg(long = "log-level", env = "LOG_LEVEL", default_value = "info")]
	pub level: LogLevel,
	#[arg(long = "log-output", env = "LOG_OUTPUT", default_value = "stdout")]
	pub output: LogOutput,
	#[arg(long = "log-format", env = "LOG_FORMAT", default_value = "json")]
	pub format: LogFmt,
	#[arg(long = "log-trace", env = "LOG_TRACE", default_value_t = false)]
	pub trace: bool,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum LogFmt {
	Plain,
	Json,
	None,
}

#[derive(Debug, Clone, ValueEnum)]
enum LogLevel {
	#[value(alias = "d")]
	Debug,
	#[value(alias = "i")]
	Info,
	#[value(alias = "w")]
	Warn,
	#[value(alias = "e")]
	Error,
}

impl From<LogLevel> for SlogLevel {
	fn from(value: LogLevel) -> Self {
		match value {
			LogLevel::Debug => DEBUG,
			LogLevel::Info => INFO,
			LogLevel::Warn => WARN,
			LogLevel::Error => ERROR,
		}
	}
}

#[derive(Clone)]
pub enum LogOutput {
	Stdout,
	Stderr,
	File(PathBuf),
}

impl FromStr for LogOutput {
	type Err = io::Error;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let output: Result<Self, io::Error> = match s.trim() {
			"stdout" | "-" => Ok(Self::Stdout),
			"stderr" | "err" => Ok(Self::Stderr),
			f => Ok(Self::File(util::valid_filepath(f, DEFAULT_LOGFILE)?)),
		};
		output
	}
}

impl Display for LogOutput {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			LogOutput::Stdout => f.write_str("stdout"),
			LogOutput::Stderr => f.write_str("stderr"),
			LogOutput::File(path) => f.write_str(path.to_str().unwrap_or_default()),
		}
	}
}
