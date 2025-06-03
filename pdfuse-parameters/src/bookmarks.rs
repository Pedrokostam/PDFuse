use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum Bookmarks{
    /// No bookmarks are added to the document.
    None,
    /// Bookmark contains only index.
    #[default]
    Index,
    /// Bookmark contains index and filename.
    IndexName,
}

impl std::fmt::Display for Bookmarks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}