use std::fmt::{self, Display};
use std::error::Error;

#[derive(Debug)]
pub struct InvalidSourceType(pub String);

impl Display for InvalidSourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for InvalidSourceType {}