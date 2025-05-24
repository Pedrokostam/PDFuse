mod indexed;
mod localization;
mod logger;
mod progress;
use std::path::PathBuf;

pub use indexed::Indexed;
pub use log;
pub use logger::CONSOLE_LOGGER;
pub use progress::{get_busy_indicator, get_progress_indicator, BusyIndicator};
pub use rust_i18n;
rust_i18n::i18n!();
/// Logs translated text (with optional arguments) as info
#[macro_export]
macro_rules! info_t {
    ($key:expr) => {{
        if $crate::log::log_enabled!($crate::log::Level::Info) {
            let translated_message = $crate::rust_i18n::t!($key);
            $crate::log::info!("{}", translated_message);
        }
    }};
    ($key:expr, $($t_args:tt)+) => {{
        if $crate::log::log_enabled!($crate::log::Level::Info) {
            let translated_message = $crate::rust_i18n::t!($key, $($t_args)*);
            $crate::log::info!("{}", translated_message);
        }
    }};
}

/// Logs translated text (with optional arguments) as debug
#[macro_export]
macro_rules! debug_t {
    ($key:expr) => {{
        if $crate::log::log_enabled!($crate::log::Level::Debug) {
            let translated_message = $crate::rust_i18n::t!($key);
            $crate::log::debug!("{}", translated_message);
        }
    }};
    ($key:expr, $($t_args:tt)+) => {{
        if $crate::log::log_enabled!($crate::log::Level::Debug) {
            let translated_message = $crate::rust_i18n::t!($key, $($t_args)*);
            $crate::log::debug!("{}", translated_message);
        }
    }};
}

/// Logs translated text (with optional arguments) as trace
#[macro_export]
macro_rules! trace_t {
    ($key:expr) => {{
        if $crate::log::log_enabled!($crate::log::Level::Trace) {
            let translated_message = $crate::rust_i18n::t!($key);
            $crate::log::trace!("{}", translated_message);
        }
    }};
    ($key:expr, $($t_args:tt)+) => {{
        if $crate::log::log_enabled!($crate::log::Level::Trace) {
            let translated_message = $crate::rust_i18n::t!($key, $($t_args)*);
            $crate::log::trace!("{}", translated_message);
        }
    }};
}

/// Logs translated text (with optional arguments) as error
#[macro_export]
macro_rules! error_t {
    ($key:expr) => {{
        if $crate::log::log_enabled!($crate::log::Level::Error) {
            let translated_message = $crate::rust_i18n::t!($key);
            $crate::log::error!("{} {}: {}", file!(), line!(), translated_message);
        }
    }};
    ($key:expr, $($t_args:tt)+) => {{
        if $crate::log::log_enabled!($crate::log::Level::Error) {
            let translated_message = $crate::rust_i18n::t!($key, $($t_args)*);
            $crate::log::error!("{} {}: {}", file!(), line!(), translated_message);
        }
    }};
}
pub fn set_localization(identifier: &str) {
    rust_i18n::set_locale(identifier);
}

/// Translates given text and passes it to a formatter
#[macro_export]
macro_rules! write_t {

    ($dst:expr, $key:expr) => {{
        let translated_message = $crate::rust_i18n::t!($key);
        write!($dst,"{}",translated_message)
    }};

    ($dst:expr, $key:expr, $($t_args:tt)+) => {{
        let translated_message = $crate::rust_i18n::t!($key, $($t_args)*);
        write!($dst,"{}",translated_message)
    }};
}

/// Creates a directory in %TEMP% and returns its path.
///
/// # Panics
///
/// Panics if it can't create the directory.
pub fn create_temp_dir() -> PathBuf {
    let temp_dir = std::env::temp_dir().join("pdfuse");
    std::fs::create_dir_all(&temp_dir).expect("Cannot create temporary directories!");
    temp_dir
}
// #[cfg(test)]
// mod tests {
//     use super::*;

// }
