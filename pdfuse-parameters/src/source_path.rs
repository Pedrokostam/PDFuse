use std::path::{Path, PathBuf};
use std::convert::TryFrom;
use std::fmt::Display;
use serde::{Serialize, Deserialize};
use walkdir::DirEntry;

use crate::file_finder::{ALL_LIBRE_EXTENSIONS, IMAGE_EXTENSIONS, PDF_EXTENSIONS};
use crate::invalid_source_type::InvalidSourceType;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Eq)]
pub enum SourcePath {
    Image(PathBuf),
    Pdf(PathBuf),
    LibreDocument(PathBuf),
}

impl PartialOrd for SourcePath {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SourcePath {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let this = match self {
            SourcePath::Image(path_buf) => path_buf,
            SourcePath::Pdf(path_buf) => path_buf,
            SourcePath::LibreDocument(path_buf) => path_buf,
        };
        let that = match other {
            SourcePath::Image(path_buf) => path_buf,
            SourcePath::Pdf(path_buf) => path_buf,
            SourcePath::LibreDocument(path_buf) => path_buf,
        };
        this.cmp(that)
    }
}

impl TryFrom<&Path> for SourcePath {
    type Error = InvalidSourceType;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        Self::from_path(value)
    }
}

impl AsRef<Path> for SourcePath {
    fn as_ref(&self) -> &Path {
        match self {
            SourcePath::Image(p) | SourcePath::LibreDocument(p) | SourcePath::Pdf(p) => p,
        }
    }
}

impl TryFrom<DirEntry> for SourcePath {
    type Error = InvalidSourceType;

    fn try_from(value: DirEntry) -> Result<Self, Self::Error> {
        match value.path().canonicalize() {
            Ok(c) => c.as_path().try_into(),
            Err(_) => Err(InvalidSourceType(display_path(value.path()))),
        }
    }
}

impl From<SourcePath> for PathBuf {
    fn from(value: SourcePath) -> Self {
        match value {
            SourcePath::Image(p) | SourcePath::LibreDocument(p) | SourcePath::Pdf(p) => p,
        }
    }
}

impl Display for SourcePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path = match self {
            Self::Image(path) | Self::Pdf(path) | Self::LibreDocument(path) => path,
        };
        write!(f, "{}", display_path(path))
    }
}

impl SourcePath {
    pub fn from_path(path: &Path) -> Result<Self, InvalidSourceType> {
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let ext_str = ext.as_str();
        let canon_path = path
            .canonicalize()
            .map_err(|_| InvalidSourceType(display_path(path)))?;
        if IMAGE_EXTENSIONS.binary_search(&ext_str).is_ok() {
            return Ok(SourcePath::Image(canon_path));
        }
        if PDF_EXTENSIONS.binary_search(&ext_str).is_ok() {
            return Ok(SourcePath::Pdf(canon_path));
        }
        if ALL_LIBRE_EXTENSIONS.binary_search(&ext_str).is_ok() {
            return Ok(SourcePath::LibreDocument(canon_path));
        }
        Err(InvalidSourceType(display_path(canon_path)))
    }
}

pub fn display_path(path: impl AsRef<Path>) -> String {
    path.as_ref()
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .to_string()
}