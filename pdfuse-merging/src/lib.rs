#[allow(dead_code)]
#[allow(unused_variables)]
//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
mod data;
mod error;
pub use data::do_everything;
pub use error::{DocumentLoadError, LibreConversionError};
rust_i18n::i18n!();
