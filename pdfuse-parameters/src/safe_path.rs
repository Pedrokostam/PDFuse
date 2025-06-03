use core::fmt;
use once_cell::sync::Lazy;
use pdfuse_utils::{debug_t, error_t};
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::ffi::OsStr;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::{env, io};

fn replace_env_var(caps: &Captures) -> String {
    let name = &caps["name"];
    env::var(name).ok().unwrap_or(caps[0].to_owned())
}

#[cfg(windows)]
static ENV_FIND: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(%(?<name>\w+)%)").expect("Regex must not fail!"));
#[cfg(not(windows))]
static ENV_FIND: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(\$(?<name>\w+))").expect("Regex must not fail!"));

#[cfg(windows)]
pub(crate) fn is_executable(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        // Check for common executable extensions
        let exe_extensions = ["exe", "bat", "cmd", "com"];
        exe_extensions
            .iter()
            .any(|&ext| extension.eq_ignore_ascii_case(ext))
    } else {
        false
    }
}
#[cfg(unix)]
pub(crate) fn is_executable(path: &Path) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        // Check if it's a file and has executable permissions
        if metadata.is_file() {
            let permissions = metadata.permissions();
            return permissions.mode() & 0o111 != 0; // Any execute bit set
        }
    }
    false
}

fn path_to_string(path: impl AsRef<Path>) -> String {
    let p = path.as_ref();
    path_to_string_impl(p)
}

fn path_to_string_impl(path: &Path) -> String {
    path.to_string_lossy()
        .trim_start_matches(r"\\?\")
        .to_string()
}

/// Replaces '~' with HOME, replaces environmental variables in the path
fn normalize_path(path: &Path) -> PathBuf {
    let Some(path_str) = path.to_str() else {
        return path.to_owned();
        // if you have a path with an invalid UTF8 character...
        // you brought this on yourself.
    };

    let replaced = ENV_FIND.replace_all(path_str, replace_env_var);

    let output: PathBuf;
    if let Some(home_dir) = dirs::home_dir() {
        if let Some(stripped) = replaced.strip_prefix('~') {
            output = home_dir.join(stripped)
        } else if let Some(stripped) = replaced.strip_prefix(r"\\?\~") {
            output = home_dir.join(stripped)
        } else {
            output = PathBuf::from(replaced.into_owned());
        }
    } else {
        output = path.to_owned();
    }
    output
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SafePath(PathBuf);

impl SafePath {
    #[inline]
    #[must_use]
    /// Converts path into human-friendly form, removing all invalid unicode characters.
    /// On Windows also remove \\?\
    pub fn to_display_string(&self) -> String {
        path_to_string(self)
    }
    pub fn new(path: impl AsRef<Path>) -> Self {
        let p = path.as_ref();
        SafePath(normalize_path(p))
    }
    #[inline]
    #[must_use]
    pub fn is_executable(&self) -> bool {
        is_executable(self)
    }
    pub fn write_to(&self, data: &[u8]) -> io::Result<()> {
        if let Some(parent) = self.parent() {
            // dont need absolute path - working dir must exists anyway
            if !parent.exists() {
                debug_t!("debug.dir_tree", path = self);
                std::fs::create_dir_all(parent)?;
            }
        }
        if self.is_dir() {
            error_t!("debug.file_is_dir", path = self);
        }
        std::fs::write(self, data)
    }
    #[inline]
    #[must_use]
    pub fn with_extension(&self, extension: impl AsRef<std::ffi::OsStr>) -> Self {
        self.0.with_extension(extension).into()
    }
    #[inline]
    #[must_use]
    pub fn join(&self, path: impl AsRef<Path>) -> Self {
        self.0.join(path).into()
    }

    pub fn get_absolute(&self) -> Result<SafePath, io::Error> {
        std::path::absolute(self).map(SafePath::from)
    }
    pub fn as_path(&self) -> &Path {
        self.as_ref()
    }

    pub fn file_name(&self) -> String {
        self.as_path()
            .file_name()
            .unwrap_or_else(|| OsStr::new("No filename"))
            .to_string_lossy()
            .to_string()
    }
}
impl Default for SafePath {
    fn default() -> Self {
        SafePath::new("")
    }
}
impl Deref for SafePath {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AsMut<PathBuf> for SafePath {
    fn as_mut(&mut self) -> &mut PathBuf {
        &mut self.0
    }
}
impl From<PathBuf> for SafePath {
    fn from(value: PathBuf) -> Self {
        SafePath::new(value)
    }
}
impl From<&Path> for SafePath {
    fn from(value: &Path) -> Self {
        SafePath::new(value)
    }
}
impl From<&str> for SafePath {
    fn from(value: &str) -> Self {
        SafePath::new(value)
    }
}

impl From<&String> for SafePath {
    fn from(value: &String) -> Self {
        SafePath::new(value)
    }
}
impl From<&std::ffi::OsStr> for SafePath {
    fn from(value: &std::ffi::OsStr) -> Self {
        SafePath::new(value)
    }
}
impl From<&clap::builder::OsStr> for SafePath {
    fn from(value: &clap::builder::OsStr) -> Self {
        SafePath::new(value)
    }
}

impl From<SafePath> for clap::builder::OsStr {
    fn from(value: SafePath) -> Self {
        value.0.as_os_str().to_owned().into()
    }
}

impl AsRef<Path> for SafePath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}
impl AsRef<std::ffi::OsStr> for SafePath {
    fn as_ref(&self) -> &std::ffi::OsStr {
        self.0.as_os_str()
    }
}
impl Borrow<Path> for SafePath {
    fn borrow(&self) -> &Path {
        &self.0
    }
}
impl fmt::Display for SafePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_display_string())
    }
}
/// Creates a directory in %TEMP% and returns its path.
///
/// # Panics
///
/// Panics if it can't create the directory.
pub fn create_temp_dir() -> SafePath {
    let temp_dir: SafePath = std::env::temp_dir().join("pdfuse").into();
    std::fs::create_dir_all(&temp_dir).expect("Cannot create temporary directories!");
    temp_dir
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn string_roundtrip() {
        let s = "test/file.xml";
        let p = SafePath::new(s);
        assert_eq!(s, p.to_display_string());
    }

    // #[test]
    // pub fn d(){
    //     let s:SafePath = env::current_exe().expect("Exe gotta be real").into();
    //     let p = SafePath::new(s);
    //     assert_eq!(s,p.to_display_string());
    // }
}
