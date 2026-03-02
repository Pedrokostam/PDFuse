use pdfuse_parameters::path::SafePath;
use pdfuse_utils::write_t;
use std::{fmt::Display, process::ExitStatus};
use thiserror::Error;

#[derive(Error)]
pub enum LibreConversionError {
    Process(#[from] std::io::Error),
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

#[derive(Debug, Error)]
pub enum DocumentLoadError {
    Io(std::io::Error),
    LibreConversion(LibreConversionError),
    InvalidFile(lopdf::Error),
    InvalidImage(ImageLoadError),
}
impl From<LibreConversionError> for DocumentLoadError {
    fn from(value: LibreConversionError) -> Self {
        Self::LibreConversion(value)
    }
}
impl From<lopdf::Error> for DocumentLoadError {
    fn from(value: lopdf::Error) -> Self {
        Self::InvalidFile(value)
    }
}
impl From<std::io::Error> for DocumentLoadError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<ImageLoadError> for DocumentLoadError {
    fn from(value: ImageLoadError) -> Self {
        Self::InvalidImage(value)
    }
}
impl Display for DocumentLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentLoadError::LibreConversion(libre_conversion_error) => {
                libre_conversion_error.fmt(f)
            }
            DocumentLoadError::InvalidFile(error) => error.fmt(f),
            DocumentLoadError::Io(error) => error.fmt(f),
            DocumentLoadError::InvalidImage(image_load_error) => image_load_error.fmt(f),
        }
    }
}

#[derive(Debug, Error)]
pub enum ImageLoadError {
    UnknownFormat(SafePath),
    UnknownPixelType(SafePath),
    UnreadableFile(#[from] std::io::Error),
}
impl Display for ImageLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageLoadError::UnknownFormat(p) => write_t!(f, "error.image_invalid_format", path = p),
            ImageLoadError::UnknownPixelType(p) => {
                write_t!(f, "error.image_invalid_pixel_type", path = p)
            },
            ImageLoadError::UnreadableFile(error) => {write!(f, "{}", error)},
        }
    }
}
#[derive(Debug)]
pub struct PageSizeParseError;
impl Display for PageSizeParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_t!(f, "error.page_parse")
    }
}
