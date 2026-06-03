use core::fmt;
use std::fmt::Display;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::str::FromStr;

use clap::{Args, ValueEnum};
pub use log_key::*;
use rslog::{
	DiscardHandler, JsonHandler, Level, Logger, TextHandler, DEBUG, ERROR, INFO,
	WARN,
};

use crate::util::valid_filepath;
#[rustfmt::skip]
pub mod log_key {
  type LogKey = &'static str;

  pub const ADDR_KEY:             LogKey = "address";
  pub const BACKUP_KEY:           LogKey = "backup";
  pub const CLEANUP_INTERVAL_KEY: LogKey = "cleanup_interval";
  pub const COMPRESSED_KEY:       LogKey = "compressed";
  pub const COUNT_KEY:            LogKey = "count";
  pub const CURRENT_KEY:          LogKey = "current";
  pub const DESC_KEY:             LogKey = "desc";
  pub const DEST_KEY:             LogKey = "destination";
  pub const DISCRETE_ID_KEY:      LogKey = "discrete_id";
  pub const ERR_KEY:              LogKey = "error_message";
  pub const EXISTS_KEY:           LogKey = "exists";
  pub const HTML_ROOT_KEY:        LogKey = "html_root";
  pub const LIMIT_KEY:            LogKey = "limit";
  pub const METHOD_KEY:           LogKey = "method";
  pub const MIME_KEY:             LogKey = "mime";
  pub const NAME_KEY:             LogKey = "name";
  pub const OLDEST_KEY:           LogKey = "oldest";
  pub const PATH_KEY:             LogKey = "path";
  pub const RANDINT_COUNT_KEY:    LogKey = "randint_count";
  pub const RETENTION_KEY:        LogKey = "retention";
  pub const RETRY_AFTER_SECS_KEY: LogKey = "retry_after_secs";
  pub const ROUTES_KEY:           LogKey = "routes";
  pub const SIZE_KEY:             LogKey = "size";
  pub const SLUG_KEY:             LogKey = "slug";
  pub const SLUG_LEN_KEY:         LogKey = "slug_len";
  pub const STORED_KEY:           LogKey = "stored";
  pub const VALUE_KEY:            LogKey = "value";
  pub const YOUNGEST_KEY:         LogKey = "youngest";
}

const DEFAULT_LOGFILE: &str = "fiche-n-chips.log";

#[derive(Args)]
#[clap(next_help_heading = "Logging")]
pub struct LoggingCfg {
	#[arg(long = "log-level", env = "LOG_LEVEL", default_value = "info")]
	/// Log verbosity level
	pub level:  LogLevel,
	#[arg(long = "log-output", env = "LOG_OUTPUT", default_value = "stdout")]
	/// Log destination
	pub output: LogOutput,
	#[arg(long = "log-format", env = "LOG_FORMAT", default_value = "json")]
	/// Log output format
	pub format: LogFmt,
	#[arg(long = "log-trace", env = "LOG_TRACE", default_value_t = false)]
	/// Enable trace logging
	pub trace:  bool,
}

macro_rules! logger_from {
	($fmt:expr, $w:expr, $lvl:expr, $trace:expr) => {
		match $fmt {
			LogFmt::Plain => {
				TextHandler::new($w).min_level($lvl).attach_source($trace).into()
			}
			LogFmt::Json => {
				JsonHandler::new($w).min_level($lvl).attach_source($trace).into()
			}
			LogFmt::None => unreachable!(),
		}
	};
}

impl TryFrom<&LoggingCfg> for Logger {
	type Error = io::Error;

	fn try_from(value: &LoggingCfg) -> Result<Self, Self::Error> {
		if matches!(value.format, LogFmt::None) {
			return Ok(DiscardHandler {}.into());
		}
		let lvl: i8 = value.level.clone().into();
		Ok(match &value.output {
			LogOutput::Stdout => {
				logger_from!(value.format, io::stdout(), lvl, value.trace)
			}
			LogOutput::Stderr => {
				logger_from!(value.format, io::stderr(), lvl, value.trace)
			}
			LogOutput::File(f) => {
				logger_from!(value.format, File::create(f)?, lvl, value.trace)
			}
		})
	}
}

#[derive(Clone, Debug, ValueEnum)]
pub enum LogFmt {
	Plain,
	Json,
	None,
}

impl Display for LogFmt {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			LogFmt::Plain => "plain",
			LogFmt::Json => "json",
			LogFmt::None => "none",
		})
	}
}

#[derive(Debug, Clone, ValueEnum)]
pub enum LogLevel {
	#[value(alias = "d")]
	Debug,
	#[value(alias = "i")]
	Info,
	#[value(alias = "w")]
	Warn,
	#[value(alias = "e")]
	Error,
}

impl From<LogLevel> for i8 {
	fn from(value: LogLevel) -> Self {
		match value {
			LogLevel::Debug => DEBUG.value(),
			LogLevel::Info => INFO.value(),
			LogLevel::Warn => WARN.value(),
			LogLevel::Error => ERROR.value(),
		}
	}
}

impl Display for LogLevel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(match self {
			LogLevel::Debug => "debug",
			LogLevel::Info => "info",
			LogLevel::Warn => "warn",
			LogLevel::Error => "error",
		})
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
			f => Ok(Self::File(valid_filepath(f, DEFAULT_LOGFILE)?)),
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
