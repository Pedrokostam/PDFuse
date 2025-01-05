use core::fmt;
use std::fmt::Display;

use colored::{ColoredString, Colorize};
use log::{Level, Metadata, Record};
pub static CONSOLE_LOGGER: ConsoleLogger = ConsoleLogger;

pub struct ConsoleLogger;
enum Message {
    Raw(String),
    Colored(ColoredString),
}
impl Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::Raw(s) => write!(f, "{}", s),
            Message::Colored(colored_string) => write!(f, "{}", colored_string),
        }
    }
}
impl From<ColoredString> for Message {
    fn from(value: ColoredString) -> Self {
        Message::Colored(value)
    }
}
impl From<String> for Message {
    fn from(value: String) -> Self {
        Message::Raw(value)
    }
}

impl log::Log for ConsoleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Debug
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let s = format!("{}", record.args());
            let colored: Message = match record.metadata().level() {
                Level::Error => s.red().into(),
                Level::Warn => s.yellow().into(),
                Level::Info => s.into(),
                Level::Debug => s.purple().into(),
                Level::Trace => s.cyan().into(),
            };
            println!("{}", colored);
        }
    }
    fn flush(&self) {}
}
