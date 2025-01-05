// pub fn set_localization(identifier:&str){
//     rust_i18n::set_locale(identifier);
// }


// /// Logs translated text (with optional arguments) as info
// #[macro_export]
// macro_rules! info_t {

//     ($key:expr) => {{
//         let translated_message = $crate::rust_i18n::t!($key);
//         $crate::log::info!("{}", translated_message);
//     }};

//     ($key:expr, $($t_args:tt)+) => {{
//         let translated_message = $crate::rust_i18n::t!($key, $($t_args)*);
//         $crate::log::info!("{}", translated_message);
//     }};
// }

// /// Logs translated text (with optional arguments) as debug
// #[macro_export]
// macro_rules! debug_t {

//     ($key:expr) => {{
//         let translated_message = $crate::rust_i18n::t!($key);
//         $crate::log::debug!("{}", translated_message);
//     }};

//     ($key:expr, $($t_args:tt)+) => {{
//         let translated_message = $crate::rust_i18n::t!($key, $($t_args)*);
//         $crate::log::debug!("{}", translated_message);
//     }};
// }

// /// Logs translated text (with optional arguments) as trace
// #[macro_export]
// macro_rules! trace_t {
    
//     ($key:expr) => {
//         let translated_message = $crate::rust_i18n::t!($key);
//         $crate::log::trace!("{}", translated_message);
//     };
    
//     ($key:expr, $($t_args:tt)+) => {
//         let translated_message = $crate::rust_i18n::t!($key, $($t_args)*);
//         $crate::log::trace!("{}", translated_message);
//     };
// }

// /// Logs translated text (with optional arguments) as error
// #[macro_export]
// macro_rules! error_t {

//     ($key:expr) => {
//         let translated_message = $crate::rust_i18n::t!($key);
//         $crate::log::error!("{}", translated_message);
//     };

//     ($key:expr, $($t_args:tt)+) => {
//         let translated_message = $crate::rust_i18n::t!($key, $($t_args)*);
//         $crate::log::error!("{}", translated_message);
//     };
// }