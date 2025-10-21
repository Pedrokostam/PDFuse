use std::{error::Error, fmt::Display, num::ParseIntError};


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
            IsoPaperError::InvalidSize(s) => write!(f,"Page size {s} is not in the valid range: [0,13]."),
            IsoPaperError::InvalidType(t) => write!(f,"Page standard {t} is not recognized."),
        }
    }
}
impl Error for IsoPaperError{}
impl From<ParseIntError> for IsoPaperError{
    fn from(_: ParseIntError) -> Self {
        IsoPaperError::NoSizeSpecified
    }
}
