use core::fmt;
use std::{
	fmt::Display,
	fs::File,
	io::{self, Write},
	path::PathBuf,
	str::FromStr,
};

use clap::{Args, ValueEnum};
use rslog::{
	DiscardHandler, JsonHandler, Level, Logger, SlogLevel, TextHandler, DEBUG,
	ERROR, INFO, WARN,
};

use crate::internal::util;

const DEFAULT_LOGFILE: &str = "/var/log/fiche-n-chips.log";

#[derive(Args)]
#[clap(next_help_heading = "Logging")]
pub struct LoggingCfg {
	#[arg(long = "log-level", env = "LOG_LEVEL", default_value = "info")]
	/// Log verbosity level
	pub level: LogLevel,
	#[arg(long = "log-output", env = "LOG_OUTPUT", default_value = "stdout")]
	/// Log destination
	pub output: LogOutput,
	#[arg(long = "log-format", env = "LOG_FORMAT", default_value = "json")]
	/// Log output format
	pub format: LogFmt,
	#[arg(long = "log-trace", env = "LOG_TRACE", default_value_t = false)]
	/// Enable trace logging
	pub trace: bool,
}

macro_rules! logger_from {
	($h:ident, $w:expr, $lvl:expr, $trace:expr) => {
		$h::new($w).min_level($lvl).attach_source($trace).into()
	};
}

impl TryFrom<LoggingCfg> for Logger {
	type Error = io::Error;
	fn try_from(value: LoggingCfg) -> Result<Self, Self::Error> {
		let lvl: i8 = value.level.into();
		Ok(match (value.format, value.output) {
			(LogFmt::Plain, LogOutput::Stdout) => {
				logger_from!(TextHandler, io::stdout(), lvl, value.trace)
			}
			(LogFmt::Plain, LogOutput::Stderr) => {
				logger_from!(TextHandler, io::stderr(), lvl, value.trace)
			}
			(LogFmt::Plain, LogOutput::File(f)) => {
				logger_from!(TextHandler, File::create(f)?, lvl, value.trace)
			}
			(LogFmt::Json, LogOutput::Stdout) => {
				logger_from!(JsonHandler, io::stdout(), lvl, value.trace)
			}
			(LogFmt::Json, LogOutput::Stderr) => {
				logger_from!(JsonHandler, io::stderr(), lvl, value.trace)
			}
			(LogFmt::Json, LogOutput::File(f)) => {
				logger_from!(JsonHandler, File::create(f)?, lvl, value.trace)
			}
			(LogFmt::None, _) => DiscardHandler {}.into(),
		})
	}
}

#[derive(Clone, Debug, ValueEnum)]
pub enum LogFmt {
	Plain,
	Json,
	None,
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
