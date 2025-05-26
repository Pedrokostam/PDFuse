use serde::{Deserialize, Serialize};

use crate::safe_path::SafePath;


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,Serialize,Deserialize)]
pub enum SafeDestination {
    File(SafePath),
    StdOut,
    StdErr,
}

impl From<SafePath> for SafeDestination {
    fn from(value: SafePath) -> Self {
        SafeDestination::File(value)
    }
}
impl From<&std::ffi::OsStr> for SafeDestination {
    fn from(value: &std::ffi::OsStr) -> Self {
        if value.to_string_lossy() == "-" {
            return SafeDestination::StdOut;
        }
        SafeDestination::File(value.into())
    }
}
impl From<&clap::builder::OsStr> for SafeDestination {
    fn from(value: &clap::builder::OsStr) -> Self {
        if value.to_string_lossy() == "-" {
            return SafeDestination::StdOut;
        }
        SafeDestination::File(value.into())
    }
}
impl From<&str> for SafeDestination{
    fn from(value: &str) -> Self {
        SafeDestination::from(std::ffi::OsStr::new(value))
    }
}

impl SafeDestination {
    pub fn write_to(&self, data: &str) -> std::io::Result<()> {
        match self {
            SafeDestination::File(safe_path) => safe_path.write_to(data.as_bytes())?,
            SafeDestination::StdOut => println!("{data}"),
            SafeDestination::StdErr => eprintln!("{data}"),
        };
        Ok(())
    }

    pub fn exists(&self)->bool{
        match self {
            SafeDestination::File(safe_path) => safe_path.exists(),
            _ => true,
        }
    }
}
