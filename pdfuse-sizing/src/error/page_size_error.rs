use std::error::Error;
use std::fmt::Display;

use crate::error::{IsoPaperError, LengthParseError, UsPaperError};

#[derive(Debug, Clone)]
pub enum PageSizeError {
    IsoPaperError(IsoPaperError),
    UsPaperError(UsPaperError),
    CustomSizeError(LengthParseError),
}

impl Display for PageSizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PageSizeError::IsoPaperError(iso_paper_error) => iso_paper_error.fmt(f),
            PageSizeError::CustomSizeError(length_parse_error) => length_parse_error.fmt(f),
            PageSizeError::UsPaperError(us_paper_error) => us_paper_error.fmt(f),
        }
    }
}

impl Error for PageSizeError {}

impl From<IsoPaperError> for PageSizeError {
    fn from(value: IsoPaperError) -> Self {
        Self::IsoPaperError(value)
    }
}

impl From<LengthParseError> for PageSizeError {
    fn from(value: LengthParseError) -> Self {
        Self::CustomSizeError(value)
    }
}

impl From<UsPaperError> for PageSizeError {
    fn from(value: UsPaperError) -> Self {
        PageSizeError::UsPaperError(value)
    }
}
