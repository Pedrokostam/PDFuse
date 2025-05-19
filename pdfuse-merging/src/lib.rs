//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
#![feature(inherent_associated_types)]
mod data;
mod error;
pub use data::load;
pub use error::{DocumentLoadError, LibreConversionError};
rust_i18n::i18n!();
