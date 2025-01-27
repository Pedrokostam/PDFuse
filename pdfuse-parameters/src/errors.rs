#[derive(Debug)]
pub struct InvalidInputFileError{}
#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Deserialization(toml::de::Error),
    Serialization(toml::ser::Error),
    InvalidFile(InvalidInputFileError)
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
impl From<InvalidInputFileError> for ConfigError{
    fn from(value: InvalidInputFileError) -> Self {
        ConfigError::InvalidFile(value)
    }
}