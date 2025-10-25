use std::fmt::Display;

use pdfuse_utils::write_t;

use crate::path::SafePath;

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Deserialization(toml::de::Error),
    Serialization(toml::ser::Error),
    NoValidFiles,
    MalformedPath(SafePath),
    MissingConfigError(SafePath),
    WhatIfMode,
}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        ConfigError::Io(value)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(value: toml::de::Error) -> Self {
        ConfigError::Deserialization(value)
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(value: toml::ser::Error) -> Self {
        ConfigError::Serialization(value)
    }
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
