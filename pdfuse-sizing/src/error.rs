use std::num::ParseIntError;
use thiserror::Error;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Error)]
pub enum LengthParseError {
    #[error(transparent)]
    InvalidUnit(#[from] UnitParseError),
    #[error("Dimension value was not specified: '{0}'")]
    NoValueSpecified(String),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Error)]
pub enum UnitParseError {
    #[error("No unit was specified")]
    NoUnitSpecified,
    #[error("Unrecognized unit: '{0}'")]
    UnrecognizedUnit(String),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Error)]
pub enum IsoPaperError {
    #[error("Not enough data to parse")]
    NotIsoPage,
    #[error("Page size is not specified")]
    NoSizeSpecified,
    #[error("Page standard is not specified: '{0}'")]
    NoTypeSpecified(String),
    #[error("Page size outside valid range: '{0}'")]
    InvalidSize(i64),
    #[error("No recognizable page standard: '{0}'")]
    InvalidType(String),
}

impl IsoPaperError {
    pub fn invalid_type<S: Into<String>>(parse_text: S) -> Self {
        Self::InvalidType(parse_text.into())
    }
    pub fn no_type_specified<S: Into<String>>(parse_text: S) -> Self {
        Self::NoTypeSpecified(parse_text.into())
    }
    pub fn invalid_size<I: Into<i64>>(value: I) -> Self {
        Self::InvalidSize(value.into())
    }
}

#[derive(Debug, Clone, Error)]
pub enum PageSizeError {
    #[error(transparent)]
    IsoPaperError(#[from] IsoPaperError),
    #[error(transparent)]
    UsPaperError(#[from] UsPaperError),
    #[error(transparent)]
    CustomSizeError(#[from] LengthParseError),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Error)]
#[error("String \"{invalid}\" could nto be matched to any valid US paper")]
pub struct UsPaperError {
    pub invalid: String,
}

impl From<ParseIntError> for IsoPaperError {
    fn from(_: ParseIntError) -> Self {
        IsoPaperError::NoSizeSpecified
    }
}
