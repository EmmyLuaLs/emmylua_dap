use structopt::StructOpt;

#[allow(unused)]
#[derive(Debug, StructOpt, Clone)]
#[structopt(name = "emmylua-dap", about = "EmmyLua Debug Adapter")]
pub struct CmdArgs {
    /// Logging level (e.g., "error", "warn", "info", "debug", "trace")
    #[structopt(long = "log-level", help = "Logging level", default_value = "info")]
    pub log_level: LogLevel,

    /// Path to the log file
    #[structopt(
        long = "log-path",
        help = "Path to the log file. Use 'none' to disable log file output.",
        default_value = ""
    )]
    pub log_path: NoneableString,
}

/// Logging level enum
#[derive(Debug, StructOpt, Clone, Copy)]
#[structopt(rename_all = "lowercase")]
pub enum LogLevel {
    /// Error level
    Error,
    /// Warning level
    Warn,
    /// Info level
    Info,
    /// Debug level
    Debug,
}

impl std::str::FromStr for LogLevel {
    type Err = String;

    fn from_str(input: &str) -> Result<LogLevel, Self::Err> {
        match input.to_lowercase().as_str() {
            "error" => Ok(LogLevel::Error),
            "warn" => Ok(LogLevel::Warn),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            _ => Err(format!(
                "Invalid log level: '{}'. Please choose 'error', 'warn', 'info', 'debug'",
                input
            )),
        }
    }
}

/// A string that can be "None" to represent an empty option
#[derive(Debug, Clone)]
pub struct NoneableString(pub Option<String>);

impl std::str::FromStr for NoneableString {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("none") {
            Ok(NoneableString(None))
        } else {
            Ok(NoneableString(Some(s.to_string())))
        }
    }
}

#[allow(unused)]
impl NoneableString {
    pub fn as_deref(&self) -> Option<&str> {
        self.0.as_deref()
    }
}
