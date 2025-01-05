mod commandline_arguments;
pub mod errors;
mod file_finder;
mod parameters;
mod paths;
pub use commandline_arguments::Args;
pub use file_finder::SourcePath;
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
