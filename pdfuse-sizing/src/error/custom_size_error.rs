use std::{error::Error, fmt::Display};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum LengthParseError {
    InvalidUnit(UnitParseError),
    NoValueSpecified,
}
impl Error for LengthParseError {}

impl From<UnitParseError> for LengthParseError {
    fn from(value: UnitParseError) -> Self {
        Self::InvalidUnit(value)
    }
}
impl Display for LengthParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LengthParseError::InvalidUnit(unit_parse_error) => unit_parse_error.fmt(f),
            LengthParseError::NoValueSpecified => write!(f, "Value is not specified"),
        }
    }
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum UnitParseError {
    NoUnitSpecified,
    UnrecognizedUnit(String),
}
impl Display for UnitParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnitParseError::NoUnitSpecified => write!(f, "No unit was specified"),
            UnitParseError::UnrecognizedUnit(u) => write!(f, "Text \"{u}\" is not a valid"),
        }
    }
}
impl Error for UnitParseError {}
