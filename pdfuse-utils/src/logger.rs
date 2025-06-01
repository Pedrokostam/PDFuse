use core::fmt;
use std::fmt::Display;

use colored::{ColoredString, Colorize};
use indicatif::{MultiProgress, ProgressBar};
use log::{Level, LevelFilter, Metadata, Record};
use once_cell::{self, sync::OnceCell};
pub static CONSOLE_LOGGER: ConsoleLogger = ConsoleLogger;

// static LOGGY: OnceCell<ConsoleLogger> = OnceCell::new();
static MULTI: OnceCell<MultiProgress> = OnceCell::new();

pub struct ConsoleLogger;
enum Message {
    Raw(String),
    Colored(ColoredString),
}
impl Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::Raw(s) => write!(f, "{s}"),
            Message::Colored(colored_string) => write!(f, "{colored_string}"),
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
            eprintln!("{colored}");
        }
    }
    fn flush(&self) {}
}

pub fn init_logger() {
    let pb = MultiProgress::new();
    if MULTI.set(pb.clone()).is_err() {
        log::error!("Logger has already been set!")
    }
    let wrap: indicatif_log_bridge::LogWrapper<ConsoleLogger> =
        indicatif_log_bridge::LogWrapper::new(pb, ConsoleLogger);
    if log::set_boxed_logger(Box::new(wrap)).is_err() {
        log::error!("Could not set bridge logger")
    };
}

pub fn register_progressbar(pb: ProgressBar) -> ProgressBar {
    MULTI
        .get()
        .expect("Bridge logger has not been set!")
        .add(pb.clone())
}
pub fn deregister_progressbar(pb: &ProgressBar) {
    MULTI
        .get()
        .expect("Bridge logger has not been set!")
        .remove(pb);
}

pub fn set_max_level(lvl: impl Into<LevelFilter>) {
    log::set_max_level(lvl.into());
}

pub fn finish_progress_bar(){
     MULTI
        .get()
        .expect("Bridge logger has not been set!")
        .clear();
}