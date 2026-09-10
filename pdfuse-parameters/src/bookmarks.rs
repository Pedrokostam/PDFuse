use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum Bookmarks{
    /// No per-file bookmark is generated. Bookmarks already present in the
    /// input documents are still preserved.
    None,
    /// A per-file bookmark containing only the index is generated.
    #[default]
    Index,
    /// A per-file bookmark containing the index and filename is generated.
    IndexName,
}

impl std::fmt::Display for Bookmarks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}