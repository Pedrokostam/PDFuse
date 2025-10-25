use std::fmt::Display;

use pdfuse_utils::write_t;
#[derive(Debug)]
pub struct MalformedPathError {
    path: String,
}

impl MalformedPathError {
    pub fn new(path: &str) -> MalformedPathError {
        MalformedPathError {
            path: path.to_owned(),
        }
    }
}

impl Display for MalformedPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_t!(f, "error.invalid_config_path", path = self.path)
    }
}
