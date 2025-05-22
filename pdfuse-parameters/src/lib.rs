mod commandline_arguments;
mod commandline_arguments_help;
pub mod errors;
pub mod file_finder;
pub mod invalid_source_type;
pub mod source_path;
pub mod commandline;
mod parameters;
mod paths;
pub use commandline_arguments::Args;
pub use parameters::{Parameters, ParametersWithPaths};
pub use source_path::SourcePath;

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
