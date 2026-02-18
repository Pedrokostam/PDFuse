use clap::builder::OsStr;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt::Display;
use std::ops::Deref;
use std::path::Path;
use walkdir::DirEntry;

use super::SafePath;
use crate::error::InvalidSourceTypeError;
use crate::file_finder::{ALL_LIBRE_EXTENSIONS, IMAGE_EXTENSIONS, PDF_EXTENSIONS};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Eq)]
pub enum SourcePath {
    Image(SafePath),
    Pdf(SafePath),
    LibreDocument(SafePath),
}

impl PartialOrd for SourcePath {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Deref for SourcePath{
    type Target = SafePath;

    fn deref(&self) -> &Self::Target {
        match self {
            SourcePath::Image(p) | SourcePath::LibreDocument(p) | SourcePath::Pdf(p) => p,
        }
    }
}

impl Ord for SourcePath {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let this_path:&SafePath = self;
        let that_path:&SafePath = other;
        this_path.cmp(that_path)
    }
}

impl TryFrom<&Path> for SourcePath {
    type Error = InvalidSourceTypeError;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        Self::try_from_path(value)
    }
}

impl AsRef<Path> for SourcePath {
    fn as_ref(&self) -> &Path {
        match self {
            SourcePath::Image(p) | SourcePath::LibreDocument(p) | SourcePath::Pdf(p) => p,
        }
    }
}

impl AsRef<SafePath> for SourcePath {
    fn as_ref(&self) -> &SafePath {
        match self {
            SourcePath::Image(p) | SourcePath::LibreDocument(p) | SourcePath::Pdf(p) => p,
        }
    }
}

impl TryFrom<DirEntry> for SourcePath {
    type Error = InvalidSourceTypeError;

    fn try_from(value: DirEntry) -> Result<Self, Self::Error> {
        match value.path().canonicalize() {
            Ok(c) => c.as_path().try_into(),
            Err(_) => Err(InvalidSourceTypeError(value.path().into())),
        }
    }
}

impl From<SourcePath> for SafePath {
    fn from(value: SourcePath) -> Self {
        match value {
            SourcePath::Image(p) | SourcePath::LibreDocument(p) | SourcePath::Pdf(p) => p,
        }
    }
}

impl TryFrom<&OsStr> for SourcePath {
    type Error = InvalidSourceTypeError;

    fn try_from(value: &OsStr) -> Result<Self, Self::Error> {
        SourcePath::try_from_path(Path::new(value))
    }
}

impl Display for SourcePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path = match self {
            Self::Image(path) | Self::Pdf(path) | Self::LibreDocument(path) => path,
        };
        write!(f, "{path}")
    }
}

impl SourcePath {
    pub fn try_from_path(path: &Path) -> Result<Self, InvalidSourceTypeError> {
        let safe_path: SafePath = path.into();
        let ext = safe_path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let ext_str = ext.as_str();
        if IMAGE_EXTENSIONS.binary_search(&ext_str).is_ok() {
            return Ok(SourcePath::Image(safe_path));
        }
        if PDF_EXTENSIONS.binary_search(&ext_str).is_ok() {
            return Ok(SourcePath::Pdf(safe_path));
        }
        if ALL_LIBRE_EXTENSIONS.binary_search(&ext_str).is_ok() {
            return Ok(SourcePath::LibreDocument(safe_path));
        }
        Err(InvalidSourceTypeError(safe_path))
    }
}
