use std::fmt::Display;

use pdfuse_utils::write_t;
use thiserror::Error;
use crate::path::SafePath;

#[derive(Debug,Error)]
pub enum ConfigError {
    Io(#[from]std::io::Error),
    Deserialization(#[from]toml::de::Error),
    Serialization(#[from]toml::ser::Error),
    NoValidFiles,
    MalformedPath(SafePath),
    MissingConfigError(SafePath),
    WhatIfMode,
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(error) => write!(f, "{error}"),
            ConfigError::Deserialization(error) => write!(f, "{error}"),
            ConfigError::Serialization(error) => write!(f, "{error}"),
            ConfigError::NoValidFiles => write_t!(f, "error.no_valid_files"),
            ConfigError::MalformedPath(path) => {
                write_t!(f, "error.invalid_config_path", path = path)
            }
            ConfigError::MissingConfigError(path) => {
                write_t!(f, "error.missing_config_file", path = path)
            }
            ConfigError::WhatIfMode => write_t!(f, "error.what_if_mode"),
        }
    }
}
