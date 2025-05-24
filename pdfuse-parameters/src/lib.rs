mod errors;
mod file_finder;
mod invalid_source_type;
mod source_path;
mod commandline;
mod commandline_help;
mod parameters;
mod paths;
pub use commandline::{Args,LogLevel,get_args};
pub use parameters::{Parameters, ParametersWithPaths};
pub use source_path::SourcePath;
pub use errors::*;

rust_i18n::i18n!();
pub use paths::path_to_string;
#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn it_works() {
    //     let result = add(2, 2);
    //     assert_eq!(result, 4);
    // }
}
