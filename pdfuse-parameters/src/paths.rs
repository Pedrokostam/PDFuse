use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use std::env;
use std::path::Path;

fn replace_env_var(caps: &Captures) -> String {
    let name = &caps["name"];
    env::var(name).ok().unwrap_or(caps[0].to_owned())
}

fn expand_vars(path: &str) -> Option<String> {
    #[cfg(unix)]
    {
        return std::fs::canonicalize(path).map(|buf| buf.to_string_lossy().into_owned())
        .ok();
    }
    static PERF_FIND: Lazy<Regex> = Lazy::new(|| Regex::new(r"(%(?<name>\w+)%)").unwrap());
    let s = PERF_FIND.replace_all(path, replace_env_var).into_owned();
    if s.contains('%') {
        None
        
    } else {
        std::path::absolute(s)
            .map(|buf| buf.to_string_lossy().into_owned())
            .ok()
    }
}

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

pub(crate) fn expand_path(path: &str) -> Option<String> {
    let non_home = if path.starts_with('~') {
        let home_dir = dirs::home_dir();
        path.replace('~', &home_dir.unwrap().to_string_lossy())
    } else {
        path.to_owned()
    };
    let output: Option<String> = expand_vars(&non_home);
    output
}


pub fn path_to_string(path: impl AsRef<Path>) -> String {
    let p = path.as_ref();
    path_to_string_impl(p)
}

fn path_to_string_impl(path:&Path)->String{
    path.to_string_lossy()
        .trim_start_matches(r"\\?\")
        .to_string()
}