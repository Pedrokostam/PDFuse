pub mod error;
mod file_finder;
pub mod path;

mod bookmarks;
mod log_level;
mod parameters;
pub use bookmarks::Bookmarks;
pub use file_finder::get_files;
pub use log_level::LogLevel;
pub use parameters::{Parameters, ParametersWithPaths};

rust_i18n::i18n!();
#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn it_works() {
    //     let result = add(2, 2);
    //     assert_eq!(result, 4);
    // }
}
