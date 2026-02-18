use crate::path::SafePath;

#[derive(Debug, thiserror::Error)]
#[error("Invalid source type: {0}")]
pub struct InvalidSourceTypeError(pub SafePath);
