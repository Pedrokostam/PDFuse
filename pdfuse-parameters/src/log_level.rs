use clap::ValueEnum;
use log::LevelFilter;
use serde::{Deserialize, Serialize};



#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogLevel {
    /// Log nothing.
    Off,
    /// Log only errors.
    Error,
    /// Log warnings and errors.
    Warn,
    /// Log everything, including debug information.
    Debug,
}

impl From<LogLevel> for LevelFilter {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Off => LevelFilter::Off,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Debug => LevelFilter::Trace,
        }
    }
}
impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}