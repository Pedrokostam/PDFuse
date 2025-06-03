mod errors;
mod file_finder;
mod invalid_source_type;
mod source_path;

mod parameters;
mod safe_path;
mod safe_destination;
mod log_level;
mod bookmarks;
pub use parameters::{Parameters, ParametersWithPaths};
pub use source_path::SourcePath;
pub use safe_path::{SafePath,create_temp_dir};
pub use safe_destination::SafeDestination;
pub use errors::*;
pub use file_finder::get_files;
pub use log_level::LogLevel;
pub use bookmarks::Bookmarks;

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
