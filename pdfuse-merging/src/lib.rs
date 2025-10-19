//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
// #![feature(inherent_associated_types)]
mod data;
mod error;
pub use data::load;
pub use error::{DocumentLoadError, LibreConversionError};
rust_i18n::i18n!();

pub(crate) fn conditional_slow_down() {
    if cfg!(feature = "slowly") {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
#[cfg(test)]
mod tests {
    use regex::Regex;
    use std::{
        io,
        path::{Path, PathBuf},
        process::{Command, Stdio},
    };
    const MEDIABOX_PATTERN: &str = r"^MediaBox.*\s(?P<W>\d+\.\d\d)\s+(?P<H>\d+\.\d\d)";
    use pdfuse_parameters::Parameters;
    use pdfuse_sizing::{IsoPaper, PageSize};

    use super::*;
    fn get_path_crate() -> PathBuf {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("data")
            .join("input.json");
        assert!(path.exists());
        path
    }
    fn get_path_workspace() -> PathBuf {
        get_path_crate().join("..")
    }
    fn run_command(path: &Path, page: i32) -> anyhow::Result<PageSize> {
        assert!(cfg!(windows));
        let exe = "pdfinfo";
        let output: std::process::Output = Command::new(exe)
            .arg(path)
            .arg("-box")
            .arg("-f")
            .arg(page.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        let output_str = String::from_utf8_lossy(&output.stdout).to_string();
        let reggy = Regex::new(MEDIABOX_PATTERN).expect("Regex dead");
        let media_captures = reggy.captures(&output_str).unwrap();
        let w = media_captures.name("W").expect("Width missing").as_str();
        let h = media_captures.name("H").expect("Height missing").as_str();
        let parse_str = format!("{}pt x {}pt", w, h);
        let size = PageSize::try_from_string(&parse_str);
        Ok(size.expect("Must be able to parse the string"))
    }
    fn get_objective_page_sizes(path: &Path, number_pages: i32) -> Vec<PageSize> {
        let mut sizes: Vec<PageSize> = vec![];
        for p in 0..number_pages {
            sizes.push(run_command(path, p).unwrap());
        }
        sizes
    }


    #[test]
    fn can_merge_pdfs() {
        let path = get_path_workspace();
        let input:Vec<(&Path,PageSize)> = vec![ ("",PageSize::Standard(IsoPaper::a(3))) ];
        let params = Parameters {
            ..Default::default()
        };
        load(vec![], &params);
    }
}

