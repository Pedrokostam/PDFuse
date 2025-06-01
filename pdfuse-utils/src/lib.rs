mod indexed;
mod localization;
mod logger;
mod progress;

pub use indexed::Indexed;
pub use log;
// use logger::CONSOLE_LOGGER;
pub use progress::{get_registered_busy_indicator, get_registered_progress_iterator,get_registered_progress_iterator_parallel, BusyIndicator};
pub use rust_i18n;
rust_i18n::i18n!();

pub use logger::{deregister_progressbar,register_progressbar,init_logger,set_max_level,finish_progress_bar};
/// Logs translated text (with optional arguments) as info
#[macro_export]
macro_rules! info_t {
    ($key:expr) => {{
        if $crate::log::log_enabled!($crate::log::Level::Info) {
            let translated_message = $crate::rust_i18n::t!($key);
            $crate::log::info!("{translated_message}");
        }
    }};
    ($key:expr, $($t_args:tt)+) => {{
        if $crate::log::log_enabled!($crate::log::Level::Info) {
            let translated_message = $crate::rust_i18n::t!($key, $($t_args)*);
            $crate::log::info!("{translated_message}");
        }
    }};
}

/// Logs translated text (with optional arguments) as debug
#[macro_export]
macro_rules! debug_t {
    ($key:expr) => {{
        if $crate::log::log_enabled!($crate::log::Level::Debug) {
            let translated_message = $crate::rust_i18n::t!($key);
            $crate::log::debug!("[d] {}:{} - {}", file!(), line!(), translated_message);
        }
    }};
    ($key:expr, $($t_args:tt)+) => {{
        if $crate::log::log_enabled!($crate::log::Level::Debug) {
            let translated_message = $crate::rust_i18n::t!($key, $($t_args)*);
            $crate::log::debug!("[d] {}:{} - {}", file!(), line!(), translated_message);
        }
    }};
}

/// Logs translated text (with optional arguments) as warning
#[macro_export]
macro_rules! warn_t {
    ($key:expr) => {{
        if $crate::log::log_enabled!($crate::log::Level::Debug) {
            let translated_message = $crate::rust_i18n::t!($key);
            $crate::log::warn!("{translated_message}");
        }
    }};
    ($key:expr, $($t_args:tt)+) => {{
        if $crate::log::log_enabled!($crate::log::Level::Debug) {
            let translated_message = $crate::rust_i18n::t!($key, $($t_args)*);
            $crate::log::warn!("{translated_message}" );
        }
    }};
}

/// Logs translated text (with optional arguments) as trace
#[macro_export]
macro_rules! trace_t {
    ($key:expr) => {{
        if $crate::log::log_enabled!($crate::log::Level::Trace) {
            let translated_message = $crate::rust_i18n::t!($key);
            $crate::log::trace!("[t] {}:{} - {}", file!(), line!(), translated_message);
        }
    }};
    ($key:expr, $($t_args:tt)+) => {{
        if $crate::log::log_enabled!($crate::log::Level::Trace) {
            let translated_message = $crate::rust_i18n::t!($key, $($t_args)*);
            $crate::log::trace!("[t] {}:{} - {}", file!(), line!(), translated_message);
        }
    }};
}

/// Logs translated text (with optional arguments) as error
#[macro_export]
macro_rules! error_t {
    ($key:expr) => {{
        if $crate::log::log_enabled!($crate::log::Level::Error) {
            let translated_message = $crate::rust_i18n::t!($key);
            $crate::log::error!("[!] {}:{} - {}", file!(), line!(), translated_message);
        }
    }};
    ($key:expr, $($t_args:tt)+) => {{
        if $crate::log::log_enabled!($crate::log::Level::Error) {
            let translated_message = $crate::rust_i18n::t!($key, $($t_args)*);
            $crate::log::error!("[!] {}:{} - {}", file!(), line!(), translated_message);
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
// #[cfg(test)]
// mod tests {
//     use super::*;

// }
