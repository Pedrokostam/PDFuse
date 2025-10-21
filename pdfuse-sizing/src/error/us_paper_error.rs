use std::{error::Error,fmt::Display};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct UsPaperError{pub invalid:String}

impl Display for UsPaperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "String {} could not be matched to any valid US paper",
            self.invalid
        )
    }
}

impl Error for UsPaperError {}

