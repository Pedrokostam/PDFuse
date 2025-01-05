use std::{error::Error, fmt::Display, num::ParseIntError};

#[derive(Debug, Clone)]
pub enum PageSizeError {
    IsoPaperError(IsoPaperError),
    CustomSizeError(LengthParseError),
}
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
impl Display for PageSizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PageSizeError::IsoPaperError(iso_paper_error) => iso_paper_error.fmt(f),
            PageSizeError::CustomSizeError(length_parse_error) => length_parse_error.fmt(f),
        }
    }
}
impl Error for PageSizeError {}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord,Debug)]
pub enum LengthParseError{
    InvalidUnit(UnitParseError),
    NoValueSpecified
}
impl Error for LengthParseError{}

impl From<UnitParseError> for LengthParseError{
    fn from(value: UnitParseError) -> Self {
        Self::InvalidUnit(value)
    }
}
impl Display for LengthParseError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{
            LengthParseError::InvalidUnit(unit_parse_error) => unit_parse_error.fmt(f),
            LengthParseError::NoValueSpecified => write!(f,"Value is not specified"),
        }
    }
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord,Debug)]
pub enum IsoPaperError{
    NotIsoPage,
    NoSizeSpecified,
    NoTypeSpecied,
    InvalidSize(i64),
    InvalidType(String),
}
impl Display for IsoPaperError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IsoPaperError::NotIsoPage => write!(f,"Not enough data to parse."),
            IsoPaperError::NoSizeSpecified => write!(f,"Page size is not specified."),
            IsoPaperError::NoTypeSpecied => write!(f,"Page standard is not specified."),
            IsoPaperError::InvalidSize(s) => write!(f,"Page size {} is not in the valid range: [0,13].",s),
            IsoPaperError::InvalidType(t) => write!(f,"Page standard {} is not recognized.",t),
        }
    }
}
impl Error for IsoPaperError{}
impl From<ParseIntError> for IsoPaperError{
    fn from(_: ParseIntError) -> Self {
        IsoPaperError::NoSizeSpecified
    }
}
#[derive(Clone,  PartialEq, Eq, PartialOrd, Ord,Debug)]
pub enum UnitParseError {
    NoUnitSpecified,
    UnrecognizedUnit(String),
}
impl Display for UnitParseError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnitParseError::NoUnitSpecified => write!(f,"No unit was specified"),
            UnitParseError::UnrecognizedUnit(u) => write!(f,"Text \"{}\" is not a valid",u),
        }
    }
}
impl Error for UnitParseError{
}
