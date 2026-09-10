use std::fmt::Display;

use pdfuse_parameters::{
    path::SourcePath,
    Bookmarks,
};

#[derive(Debug, Clone)]
pub enum DocumentSources {
    Single(SourcePath),
    Multiple(Vec<SourcePath>),
}

impl DocumentSources {
    pub fn new_single(path: impl Into<SourcePath>) -> Self {
        DocumentSources::Single(path.into())
    }
    pub fn new_multi(paths: impl IntoIterator<Item = impl Into<SourcePath>>) -> Self {
        let veccy: Vec<SourcePath> = paths.into_iter().map(|x| x.into()).collect();
        if veccy.is_empty() {
            panic!("Created multisource document with no sources!");
        }
        let disc = std::mem::discriminant(&veccy[0]);
        if !veccy.iter().all(|x| std::mem::discriminant(x) == disc) {
            panic!("Created multisource document with different source types!");
        }
        DocumentSources::Multiple(veccy)
    }

    /// True when this document bundles several source files (image runs), in
    /// which case each source maps to one page. `Single` sources (a PDF or a
    /// lone image) map one title to the whole file's first page.
    pub(crate) fn is_multi(&self) -> bool {
        matches!(self, DocumentSources::Multiple(_))
    }

    pub fn get_source_bookmarks(&self, option: Bookmarks, starting_index: usize) -> Vec<String> {
        match option {
            Bookmarks::None => vec![],
            Bookmarks::Index => match &self {
                DocumentSources::Single(_) => vec![starting_index.to_string()],
                DocumentSources::Multiple(safe_paths) => (0..safe_paths.len())
                    .map(|x| (x + starting_index).to_string())
                    .collect(),
            },
            Bookmarks::IndexName => match &self {
                DocumentSources::Single(safe_path) => vec![format!(
                    "{i} - {n}",
                    i = starting_index,
                    n = safe_path.file_name()
                )],
                DocumentSources::Multiple(safe_paths) => safe_paths
                    .iter()
                    .enumerate()
                    .map(|p| format!("{i} - {n}", i = p.0 + starting_index, n = p.1.file_name()))
                    .collect(),
            },
        }
    }
}

impl Display for DocumentSources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentSources::Single(source_path) => write!(f, "{source_path}"),
            DocumentSources::Multiple(source_paths) => write!(f, "{} sources", source_paths.len()),
        }
    }
}
