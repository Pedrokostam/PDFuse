use std::fmt::Display;

use pdfuse_parameters::{path::SafePath, Bookmarks};

#[derive(Debug, Clone)]
pub enum DocumentSources {
    Single(SafePath),
    Multiple(Vec<SafePath>),
}

impl DocumentSources {
    pub fn new_single(path: impl Into<SafePath>) -> Self {
        DocumentSources::Single(path.into())
    }
    pub fn new_multi(paths: impl IntoIterator<Item = impl Into<SafePath>>) -> Self {
        DocumentSources::Multiple(paths.into_iter().map(|x| x.into()).collect())
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
            DocumentSources::Single(safe_path) => write!(f, "{safe_path}"),
            DocumentSources::Multiple(safe_paths) => write!(f, "{} sources", safe_paths.len()),
        }
    }
}

impl From<Vec<SafePath>> for DocumentSources {
    fn from(value: Vec<SafePath>) -> Self {
        DocumentSources::Multiple(value)
    }
}
