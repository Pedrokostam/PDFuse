use std::error::Error;
use std::fmt::{self, Display};

use crate::path::SafePath;

#[derive(Debug)]
pub struct InvalidSourceTypeError(pub SafePath);

impl Display for InvalidSourceTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for InvalidSourceTypeError {}

