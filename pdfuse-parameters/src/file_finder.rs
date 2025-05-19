use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use std::{fmt::Display, path::Path};

use indicatif::ProgressBar;
use once_cell::sync::Lazy;
use pdfuse_utils::{error_t, get_busy_indicator, BusyIndicator};
use pdfuse_utils::{info_t, Indexed};
use serde::Deserialize;
use serde::Serialize;
use walkdir::{DirEntry, WalkDir};

pub(crate) const IMAGE_EXTENSIONS: &[&str] = &[
    "bmp", "jpeg", "jp2", "jpg", "jpx", "jxr", "pam", "pbm", "pnm", "png", "psd", "tiff",
];

pub(crate) const DRAWING_EXTENSIONS: &[&str] = &[
    "cdr", "odg", "otg", "pub", "std", "svg", "sxd", "vdx", "vsd", "vsdm", "vsdx",
];

pub(crate) const PRESENTATION_EXTENSIONS: &[&str] = &[
    "dps", "dpt", "fodp", "odp", "otp", "pot", "potm", "potx", "pps", "ppsx", "ppt", "pptx", "sxd",
    "sti", "xml",
];

pub(crate) const CALC_EXTENSIONS: &[&str] = &[
    "123", "csv", "dif", "et", "ett", "fods", "ods", "ots", "sxc", "stc", "wb2", "wk1", "wks",
    "xlc", "xlk", "xlm", "xls", "xlsb", "xlt", "xltm", "xltx", "xlw", "xlsx", "xml",
];

pub(crate) const TEXT_EXTENSIONS: &[&str] = &[
    "doc", "docm", "docx", "dot", "dotm", "dotx", "fodt", "htm", "html", "hwp", "lwp", "odm",
    "odt", "oth", "ott", "psw", "rtf", "stw", "sxw", "txt", "wpd", "wpt", "wps", "xhtml", "xml",
];

pub(crate) const PDF_EXTENSIONS: &[&str] = &["pdf"];

use crate::source_path::SourcePath;

pub(crate) static ALL_SIMPLE_SUPPORTED_EXTENSIONS: Lazy<Vec<&str>> =
    Lazy::new(|| [IMAGE_EXTENSIONS, PDF_EXTENSIONS].concat());

pub(crate) static ALL_LIBRE_EXTENSIONS: Lazy<Vec<&str>> = Lazy::new(|| {
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

pub(crate) static ALL_SUPPORTED_EXTENSIONS: Lazy<Vec<&str>> = Lazy::new(|| {
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

fn display_path(path: impl AsRef<Path>) -> String {
    path.as_ref()
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .to_string()
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
    busy_indicator: &ProgressBar,
) {
    // let path = folder_path.replace('\\', "/");
    let extensions = if allow_office_docs {
        &ALL_SUPPORTED_EXTENSIONS
    } else {
        &ALL_SIMPLE_SUPPORTED_EXTENSIONS
    };
    let mut count = output.len();
    let enumerable = WalkDir::new(folder_path)
        .max_depth(max_depth)
        .into_iter()
        .filter_map(|e| e.ok()) // Skip errors
        .filter(|entry| is_valid_source(entry, extensions))
        .filter_map(|d| SourcePath::try_from(d).ok())
        .inspect(|_| {
            count += 1;
            busy_indicator.set_message(format!("Found {count} files"));
        });
    output.extend(enumerable);
}
pub fn get_files(
    paths: &[impl AsRef<Path>],
    max_depth: usize,
    allow_office_docs: bool,
    sort: bool,
) -> Vec<Indexed<SourcePath>> {
    let busy = get_busy_indicator();
    let mut valid_paths: Vec<SourcePath> = vec![];
    for path in paths.iter() {
        let path = path.as_ref();
        if path.is_file() {
            match SourcePath::from_path(path) {
                Ok(source_path) => {
                    valid_paths.push(source_path);
                    busy.set_message(format!("Found {} files", valid_paths.len()));
                }
                // only report the error if it was specified directly in the commandline
                // do not do it for files from recursion
                Err(err) => error_t!("error.not_supported", path = err),
            };
        } else if path.is_dir() {
            recurse_folder(path, max_depth, allow_office_docs, &mut valid_paths, &busy)
        }
    }
    if sort {
        valid_paths.sort();
    }
    drop(busy);
    info_t!("found_files_header");
    let files: Vec<Indexed<SourcePath>> = valid_paths
        .into_iter()
        .enumerate()
        .map(|tup| tup.into())
        .collect();
    if files.is_empty() {
        return vec![];
    }

    let width = (files.len().ilog10() + 1) as usize;
    for file in &files {
        let padded = format!("{:width$}", file.index() + 1, width = width);
        info_t!("found_file", path = file.value(), index = padded);
    }
    files
}
