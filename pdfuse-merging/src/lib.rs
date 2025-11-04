//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
// #![feature(inherent_associated_types)]
pub mod data;
// mod documenter;
pub mod error;
pub use data::load;
rust_i18n::i18n!();

pub(crate) fn conditional_slow_down() {
    if cfg!(feature = "slowly") {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
#[cfg(test)]
mod tests {
    use pdfuse_utils::Indexed;
    use regex::Regex;
    use std::{
        env, fs,
        path::{Path, PathBuf},
        process::{Command, Stdio},
    };
    const MEDIABOX_PATTERN: &str = r"^MediaBox.*\s(?P<W>\d+\.\d\d)\s+(?P<H>\d+\.\d\d)";
    use pdfuse_parameters::{path::{SafePath, SourcePath}, Parameters };
    use pdfuse_sizing::{
        paper::{CustomPage, IsoPaper, Page, UsPaper},
        TransposableSize,
    };

    use super::*;
    fn get_path_crate() -> SafePath {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("data")
            .join("input.json");
        assert!(path.exists());
        path.into()
    }
    fn get_path_workspace() -> SafePath {
        get_path_crate().join("..").into()
    }
    fn run_command(path: &Path, page: usize) -> anyhow::Result<Page> {
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
        let size = Page::try_from_string(&parse_str);
        Ok(size.expect("Must be able to parse the string"))
    }
    // fn get_objective_page_sizes(path: &Path, number_pages: i32) -> Vec<Page> {
    //     let mut sizes: Vec<Page> = vec![];
    //     for p in 0..number_pages {
    //         sizes.push(run_command(path, p).unwrap());
    //     }
    //     sizes
    // }

    #[test]
    fn can_merge_pdfs() {
        let temp: SafePath = env::temp_dir().join("can_merge_pdfs_test.pdf").into();
        if fs::exists(&temp).expect("Can't check file") {
            fs::remove_file(&temp);
        }
        let path = get_path_workspace().join("test_items").join("single_pdfs");
        let input: Vec<(&str, Page)> = vec![
            (
                "100x100cm.pdf",
                Page::Custom(CustomPage::from_centimeters(100, 100)),
            ),
            ("A3.pdf", Page::Standard(IsoPaper::a(3))),
            (
                "A3Transposed.pdf",
                Page::Standard(IsoPaper::a_transposed(3)),
            ),
            ("A4.pdf", Page::Standard(IsoPaper::a(4))),
            (
                "A4Transposed.pdf",
                Page::Standard(IsoPaper::a_transposed(4)),
            ),
            ("Letter.pdf", Page::American(UsPaper::Letter)),
            (
                "LetterTransposed.pdf",
                Page::American(UsPaper::Letter).transposed(),
            ),
            (
                "Lorem30x10cm.pdf",
                Page::Custom(CustomPage::from_centimeters(30, 10)),
            ),
        ];
        let source: Vec<(Indexed<SourcePath>, Page)> = input
            .into_iter()
            .enumerate()
            .map(|enumpair| {
                (
                    Indexed::new(enumpair.0, SourcePath::Pdf(path.join(enumpair.1 .0))),
                    enumpair.1 .1,
                )
            })
            .collect();
        let params = Parameters {
            output_file: temp.clone(),
            ..Default::default()
        };
        load(source.iter().map(|x| x.0.clone()).collect(), &params);
        for t in source {
            let size = run_command(temp.as_path(), t.0.index()).expect("Can't parse page size");
            assert_eq!(
                size,
                t.1,
                "Parsed size of {} does not equal {} (it's {})",
                t.0.file_name(),
                t.1,
                size
            );
        }
    }
}
