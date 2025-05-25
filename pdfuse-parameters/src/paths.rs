use serde::{Deserialize, Serialize};
use core::fmt;
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use std::borrow::Borrow;
use std::ffi::OsString;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::{env, io};

fn replace_env_var(caps: &Captures) -> String {
    let name = &caps["name"];
    env::var(name).ok().unwrap_or(caps[0].to_owned())
}

// fn expand_vars(path: &str) -> Option<String> {
//     #[cfg(unix)]
//     {
//         return std::fs::canonicalize(path)
//             .map(|buf| buf.to_string_lossy().into_owned())
//             .ok();
//     }
//     static PERF_FIND: Lazy<Regex> = Lazy::new(|| Regex::new(r"(%(?<name>\w+)%)").unwrap());
//     let s = PERF_FIND.replace_all(path, replace_env_var).into_owned();
//     if s.contains('%') {
//         None
//     } else {
//         std::path::absolute(s)
//             .map(|buf| buf.to_string_lossy().into_owned())
//             .ok()
//     }
// }
#[cfg(windows)]
static ENV_FIND: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(%(?<name>\w+)%)").expect("Regex must not fail!"));
#[cfg(not(windows))]
static ENV_FIND: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(\$(?<name>\w+))").expect("Regex must not fail!"));

#[cfg(windows)]
pub(crate) fn is_executable(path: &str) -> bool {
    let path = Path::new(path);
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

// pub(crate) fn expand_path(path: &str) -> Option<String> {
//     let non_home = if path.starts_with('~') {
//         let home_dir = dirs::home_dir();
//         path.replace('~', &home_dir.unwrap().to_string_lossy())
//     } else {
//         path.to_owned()
//     };
//     let output: Option<String> = expand_vars(&non_home);
//     output
// }

fn path_to_string(path: impl AsRef<Path>) -> String {
    let p = path.as_ref();
    path_to_string_impl(p)
}

fn path_to_string_impl(path: &Path) -> String {
    path.to_string_lossy()
        .trim_start_matches(r"\\?\")
        .to_string()
}

// pub fn path_from_os_string(os_str: &OsStr) -> Result<PathBuf, String> {
//     let path = os_str.to_string_lossy();
//     let expanded = expand_path(&path);
// }

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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,Serialize,Deserialize)]
pub struct SafePath(PathBuf);

impl SafePath {
    /// Converts path into human-friendly form, removing all invalid unicode characters.
    /// On Windows also remove \\?\
    pub fn to_display_string(&self) -> String {
        path_to_string(self)
    }
    pub fn new(path: impl AsRef<Path>) -> Self {
        let p = path.as_ref();
        SafePath(normalize_path(p))
    }
    pub fn is_executable(&self) -> bool {
        is_executable(&self.to_display_string())
    }
    pub fn write_to(&self, data: &[u8]) -> io::Result<()> {
        std::fs::create_dir_all(self)?;
        std::fs::write(self, data)
    }
}
impl Default for SafePath{
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
// impl From<SafePath> for &OsStr{
//     fn from(value: SafePath) -> Self {
//         value.0.as_os_str()
//     }
// }

impl From<SafePath> for clap::builder::OsStr{
    fn from(value: SafePath) -> Self {
        value.0.as_os_str().to_owned().into()
    }
}

impl AsRef<Path> for SafePath {
    fn as_ref(&self) -> &Path {
        &self.0
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
        return SafeDestination::File(value.into());
    }
}
impl From<&clap::builder::OsStr> for SafeDestination {
    fn from(value: &clap::builder::OsStr) -> Self {
        if value.to_string_lossy() == "-" {
            return SafeDestination::StdOut;
        }
        return SafeDestination::File(value.into());
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
