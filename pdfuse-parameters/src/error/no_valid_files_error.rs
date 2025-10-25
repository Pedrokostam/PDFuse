use std::fmt::Display;

use pdfuse_utils::write_t;

#[derive(Debug)]
pub struct NoValidFilesError {}
impl Display for NoValidFilesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_t!(f, "error.no_valid_files")
    }
}
