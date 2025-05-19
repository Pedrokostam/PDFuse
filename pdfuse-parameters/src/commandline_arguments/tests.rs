use std::fmt::Display;
use std::path::PathBuf;

fn get_changed_args() -> Args {
    let def: Args = Args::default();
    Args {
        files: vec![],
        save_config: None,
        confirm_exit: !def.confirm_exit,
        quiet: !def.quiet,
        what_if: !def.what_if,
        language: Some("de".to_owned()),
        config: None,
        recursion_limit: 17,
        image_page_fallback_size: IsoPaper::c(10).into(),
        dpi: 1337,
        quality:13,
        margin: CustomSize::from_inches(0.5, 0.5),
        force_image_page_fallback_size: !def.force_image_page_fallback_size,
        alphabetic_file_sorting: !def.alphabetic_file_sorting,
        libreoffice_path: vec!["/usr/bin/sl".to_owned()],
        output_directory: "~/o".to_owned(),
        output_file: Some("~/o/p.pdf".to_owned()),
    }
}

macro_rules! fake_args {
    ($($item:expr),* $(,)?) =>{ {
        let mut a:Vec<String> = vec![
            "PROGRAM_PLACEHOLDER".to_owned(),
            "FILE_PLACEHOLDER".to_owned(),
        ];
        $(
           a.push($item.to_string());
        )*
        Some(a)
    }};
}
#[derive(Debug, Clone)]
struct Diff(String, String, String);
impl Display for Diff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} vs. {}", self.0, self.1, self.2)
    }
}

macro_rules! changes {
    ($a:expr,$b:expr,$($item:ident),* $(,)?) =>{ {
        let mut a:Vec<Diff> = vec![
        ];
        $(
            let item_a=$a.$item.clone() ;
            let item_b= $b.$item.clone();
        if(item_a != item_b){
            a.push(Diff(stringify!($item).to_owned(),format!("{:?}",item_a),format!("{:?}",item_b)));
        }
        )*
        a
    }};
}
fn cat(v: &[Diff]) -> String {
    v.iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join("; ")
}

fn find_changes(a: &Args, b: &Args) -> Vec<Diff> {
    changes!(
        a,
        b,
        confirm_exit,
        quiet,
        what_if,
        language,
        recursion_limit,
        image_page_fallback_size,
        dpi,
        margin,
    )
}

use super::*;
fn get_path_test_config() -> String {
    get_path("test_resources/test_config.toml")
}
fn get_path(path_end: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(path_end);
    path.to_string_lossy().into_owned()
}
#[test]
fn commandline_args_triumph_over_loaded() {
    let default = Args::default();
    let config_path = get_path_test_config();
    let loaded_config = Args::load_config_file(&config_path).unwrap();
    let commandline = Args::create_from(fake_args![
        "--dpi",
        &default.dpi,
        "--margin",
        &default.margin,
        "--config",
        &config_path,
    ]).unwrap();
    assert_ne!(
        default.dpi, loaded_config.dpi,
        "DPI is the same in default and loaded"
    );
    assert_eq!(commandline.dpi, default.dpi, "Command-line DPI was ignored");
    assert_ne!(
        default.margin, loaded_config.margin,
        "Margin is the same in default and loaded"
    );
    assert_eq!(
        commandline.margin, default.margin,
        "Command-line margin was ignored"
    );
}
#[test]
fn loaded_args_triumph_over_default() {
    let default = Args::default();
    let config_path = get_path_test_config();
    let loaded_config = Args::load_config_file(&config_path).unwrap();
    let commandline = Args::create_from(fake_args!["--config", &config_path]).unwrap();
    assert_ne!(
        default.dpi, loaded_config.dpi,
        "DPI is the same in default and loaded"
    );
    assert_eq!(commandline.dpi, loaded_config.dpi, "Loaded DPI was ignored");
    assert_ne!(
        default.margin, loaded_config.margin,
        "Margin is the same in default and loaded"
    );
    assert_eq!(
        commandline.margin, loaded_config.margin,
        "Loaded margin was ignored"
    );
}
#[test]
fn implicit_config_load() {
    let config_path = get_path_test_config();
    let implicit_path = get_path(DEFAULT_CONFIG_PATH);
    fs::copy(config_path, &implicit_path);
    let default = Args::default();
    let a = Args::create_from(fake_args![]).unwrap();
    fs::remove_file(&implicit_path);
    assert_ne!(a.dpi, default.dpi);
    assert_ne!(a.margin, default.margin);
}
