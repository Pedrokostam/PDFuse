use std::path::PathBuf;
use std::time::Duration;
use std::{fmt::Display, path::Path};

use once_cell::sync::Lazy;
use pdfuse_utils::{error_t, BusyIndicator};
use pdfuse_utils::{info_t, Indexed};
use serde::Deserialize;
use serde::Serialize;
use walkdir::{DirEntry, WalkDir};

const IMAGE_EXTENSIONS: &[&str] = &[
    "bmp", "jpeg", "jp2", "jpg", "jpx", "jxr", "pam", "pbm", "pnm", "png", "psd", "tiff",
];

const DRAWING_EXTENSIONS: &[&str] = &[
    "cdr", "odg", "otg", "pub", "std", "svg", "sxd", "vdx", "vsd", "vsdm", "vsdx",
];

const PRESENTATION_EXTENSIONS: &[&str] = &[
    "dps", "dpt", "fodp", "odp", "otp", "pot", "potm", "potx", "pps", "ppsx", "ppt", "pptx", "sxd",
    "sti", "xml",
];

const CALC_EXTENSIONS: &[&str] = &[
    "123", "csv", "dif", "et", "ett", "fods", "ods", "ots", "sxc", "stc", "wb2", "wk1", "wks",
    "xlc", "xlk", "xlm", "xls", "xlsb", "xlt", "xltm", "xltx", "xlw", "xlsx", "xml",
];

const TEXT_EXTENSIONS: &[&str] = &[
    "doc", "docm", "docx", "dot", "dotm", "dotx", "fodt", "htm", "html", "hwp", "lwp", "odm",
    "odt", "oth", "ott", "psw", "rtf", "stw", "sxw", "txt", "wpd", "wpt", "wps", "xhtml", "xml",
];

const PDF_EXTENSIONS: &[&str] = &["pdf"];

static ALL_SIMPLE_SUPPORTED_EXTENSIONS: Lazy<Vec<&str>> =
    Lazy::new(|| [IMAGE_EXTENSIONS, PDF_EXTENSIONS].concat());

static ALL_LIBRE_EXTENSIONS: Lazy<Vec<&str>> = Lazy::new(|| {
    let mut s = [
        DRAWING_EXTENSIONS,
        PRESENTATION_EXTENSIONS,
        CALC_EXTENSIONS,
        TEXT_EXTENSIONS,
    ]
    .concat();
    s.sort();
    s
});

static ALL_SUPPORTED_EXTENSIONS: Lazy<Vec<&str>> = Lazy::new(|| {
    [
        ALL_SIMPLE_SUPPORTED_EXTENSIONS.as_slice(),
        ALL_LIBRE_EXTENSIONS.as_slice(),
    ]
    .concat()
});

fn is_valid_source(entry: &DirEntry, extensions: &[&str]) -> bool {
    match entry.path().extension() {
        None => false,
        Some(ext) => {
            let ext = ext.to_string_lossy().to_lowercase(); // Convert to lowercase
            extensions.iter().any(|&e| e.eq_ignore_ascii_case(&ext))
        }
    }
}

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
pub struct InvalidSourceType(String);

impl Display for InvalidSourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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

fn display_path(path: impl AsRef<Path>) -> String {
    path.as_ref()
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .to_string()
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

// #[derive(Clone, PartialEq, Debug, Eq, PartialOrd, Ord)]
// pub struct IndexedSourcePath {
//     pub index: usize,
//     pub source: SourcePath,
// }

// unsafe impl Send for IndexedSourcePath {}
// impl Display for IndexedSourcePath {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.source)
//     }
// }
// impl From<(usize, SourcePath)> for IndexedSourcePath {
//     fn from(value: (usize, SourcePath)) -> Self {
//         IndexedSourcePath {
//             index: value.0,
//             source: value.1,
//         }
//     }
// }

/// Recursively iterates over a directory up to a specified depth
fn recurse_folder(
    folder_path: impl AsRef<Path>,
    max_depth: usize,
    allow_office_docs: bool,
    output: &mut Vec<SourcePath>,
) {
    // let path = folder_path.replace('\\', "/");
    let extensions = if allow_office_docs {
        &ALL_SUPPORTED_EXTENSIONS
    } else {
        &ALL_SIMPLE_SUPPORTED_EXTENSIONS
    };
    let enumerable = WalkDir::new(folder_path)
        .max_depth(max_depth)
        .into_iter()
        .filter_map(|e| e.ok()) // Skip errors
        .filter(|entry| is_valid_source(entry, extensions))
        .filter_map(|d| SourcePath::try_from(d).ok());
    for index_source in enumerable {
        output.push(index_source);
        // if let Some(x) = &params.callback {
        //     x.update(output.len())
        // }
    }
}
pub fn get_files(
    paths: &[impl AsRef<Path>],
    max_depth: usize,
    allow_office_docs: bool,
    sort: bool,
) -> Vec<Indexed<SourcePath>> {
    let busy = BusyIndicator::new();
    let mut valid_paths: Vec<SourcePath> = vec![];
    for path in paths.iter() {
        let path = path.as_ref();
        if path.is_file() {
            match SourcePath::from_path(path) {
                Ok(source_path) => valid_paths.push(source_path),
                // only report the error if it was specified directly in the commandline
                // do not do it for files from recursion
                Err(err) => error_t!("error.not_supported", path = err),
            };
        } else if path.is_dir() {
            recurse_folder(path, max_depth, allow_office_docs, &mut valid_paths)
        }
    }
    if sort {
        valid_paths.sort();
    }
    std::thread::sleep(Duration::from_millis(2000));
    drop(busy);
    info_t!("found_files_header");
    for val_path in &valid_paths {
        info_t!("found_file", path = val_path);
    }
    valid_paths
        .into_iter()
        .enumerate()
        .map(|tup| tup.into())
        .collect()
}
