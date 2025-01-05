#![allow(dead_code)]
#![allow(unused)]
mod parameters;
mod commandline_arguments;
mod paths;
mod file_finder;
pub mod errors;
pub use parameters::{ParametersWithPaths,Parameters};
pub use file_finder::{SourcePath};
pub use commandline_arguments::Args;
rust_i18n::i18n!();

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn it_works() {
    //     let result = add(2, 2);
    //     assert_eq!(result, 4);
    // }
}
