use std::{fmt::Display, process::ExitStatus};

pub enum LibreConversionError {
    Process(std::io::Error),
    Status(ExitStatus),
}
impl std::fmt::Debug for LibreConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Process(arg0) => f.debug_tuple("Process").field(arg0).finish(),
            Self::Status(arg0) => f.debug_tuple("Status").field(arg0).finish(),
        }
    }
}
impl Display for LibreConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LibreConversionError::Process(error) => error.fmt(f),
            LibreConversionError::Status(exit_status) => exit_status.fmt(f),
        }
    }
}
impl From<std::io::Error> for LibreConversionError {
    fn from(value: std::io::Error) -> Self {
        Self::Process(value)
    }
}

#[derive(Debug)]
pub enum DocumentLoadError {
    Io(std::io::Error),
    LibreConversion(LibreConversionError),
    InvalidFile(printpdf::lopdf::Error),
}
impl From<LibreConversionError> for DocumentLoadError {
    fn from(value: LibreConversionError) -> Self {
        Self::LibreConversion(value)
    }
}
impl From<printpdf::lopdf::Error> for DocumentLoadError {
    fn from(value: printpdf::lopdf::Error) -> Self {
        Self::InvalidFile(value)
    }
}
impl From<std::io::Error> for DocumentLoadError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
impl Display for DocumentLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentLoadError::LibreConversion(libre_conversion_error) => libre_conversion_error.fmt(f),
            DocumentLoadError::InvalidFile(error) => error.fmt(f),
            DocumentLoadError::Io(error) => error.fmt(f),
        }
    }
}
