mod localization;
mod logger;
mod progress;
mod indexed;
use std::path::PathBuf;

pub use log;
pub use indexed::Indexed;
pub use logger::CONSOLE_LOGGER;
pub use progress::BusyIndicator;
pub use rust_i18n;
rust_i18n::i18n!();
/// Logs translated text (with optional arguments) as info
#[macro_export]
macro_rules! info_t {

    ($key:expr) => {{
        let translated_message = $crate::rust_i18n::t!($key);
        $crate::log::info!("{}", translated_message);
    }};

    ($key:expr, $($t_args:tt)+) => {{
        let translated_message = $crate::rust_i18n::t!($key, $($t_args)*);
        $crate::log::info!("{}", translated_message);
    }};
}

/// Logs translated text (with optional arguments) as debug
#[macro_export]
macro_rules! debug_t {

    ($key:expr) => {{
        let translated_message = $crate::rust_i18n::t!($key);
        $crate::log::debug!("{}", translated_message);
    }};

    ($key:expr, $($t_args:tt)+) => {{
        let translated_message = $crate::rust_i18n::t!($key, $($t_args)*);
        $crate::log::debug!("{}", translated_message);
    }};
}

/// Logs translated text (with optional arguments) as trace
#[macro_export]
macro_rules! trace_t {

    ($key:expr) => {{
        let translated_message = $crate::rust_i18n::t!($key);
        $crate::log::trace!("{}", translated_message);
    }};

    ($key:expr, $($t_args:tt)+) => {{
        let translated_message = $crate::rust_i18n::t!($key, $($t_args)*);
        $crate::log::trace!("{}", translated_message);
    }};
}

/// Logs translated text (with optional arguments) as error
#[macro_export]
macro_rules! error_t {

    ($key:expr) => {{
        let translated_message = $crate::rust_i18n::t!($key);
        $crate::log::error!("{}", translated_message);
    }};

    ($key:expr, $($t_args:tt)+) => {{
        let translated_message = $crate::rust_i18n::t!($key, $($t_args)*);
        $crate::log::error!("{}", translated_message);
    }};
}
pub fn set_localization(identifier: &str) {
    rust_i18n::set_locale(identifier);
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
