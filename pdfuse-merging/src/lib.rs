//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
// #![feature(inherent_associated_types)]
mod data;
mod error;
pub use data::load;
pub use error::{DocumentLoadError, LibreConversionError};
rust_i18n::i18n!();

pub(crate) fn conditional_slow_down() {
    if cfg!(feature = "slowly") {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}