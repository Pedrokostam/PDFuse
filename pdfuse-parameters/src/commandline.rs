use std::fmt::Display;
use std::path::PathBuf;

use crate::ParametersWithPaths;
use clap::builder::{styling, OsStr, Str};
use clap::error::ErrorKind;
use clap::{
    arg, command, value_parser, Arg, ArgAction, ArgGroup, ArgMatches, Command, ValueEnum, ValueHint,
};
use indoc::indoc;
use pdfuse_sizing::{CustomSize, IsoPaper, PageSize};
use serde::{Deserialize, Serialize};
use crate::commandline_arguments_help;

const DEFAULT_CONFIG_PATH: &str = "config_auto.toml";

const STYLES: styling::Styles = styling::Styles::styled()
    .header(styling::AnsiColor::Magenta.on_default().bold().italic())
    .usage(styling::AnsiColor::BrightMagenta.on_default().bold())
    .literal(styling::AnsiColor::Green.on_default().bold())
    .placeholder(styling::AnsiColor::Cyan.on_default());

const DEFAULT_LIBRE_PATHS: &[&str] = {
    #[cfg(windows)]
    {
        &[
            r"%PROGRAMFILES%\LibreOffice\program\soffice.exe",
            r"%PROGRAMFILES(X86)%\LibreOffice\program\soffice.exe",
        ]
    }
    #[cfg(unix)]
    {
        &["/usr/bin/soffice"]
    }
};

fn get_default_libre() -> Vec<String> {
    DEFAULT_LIBRE_PATHS.iter().map(|p| p.to_string()).collect()
}

#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogLevel {
    /// Log only errors.
    Error,
    /// Log warnings and errors.
    Warn,
    /// Log everything, including debug information.
    Debug,
}

impl From<LogLevel> for log::LevelFilter {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Debug => log::LevelFilter::Trace,
        }
    }
}
impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Args {
    pub files: Vec<String>,
    pub save_config: Option<String>,
    pub confirm_exit: bool,
    pub what_if: bool,
    pub language: Option<String>,
    pub config: Option<String>,
    pub recursion_limit: usize,
    pub image_page_fallback_size: PageSize,
    pub dpi: u16,
    pub quality: u8,
    pub lossless: bool,
    pub log: LogLevel,
    pub margin: CustomSize,
    pub force_image_page_fallback_size: bool,
    pub alphabetic_file_sorting: bool,
    pub libreoffice_path: Vec<String>,
    pub output_directory: String,
    pub output_file: Option<String>,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            files: Default::default(),
            save_config: None,
            confirm_exit: false,
            what_if: false,
            language: None,
            config: None,
            recursion_limit: 4,
            image_page_fallback_size: IsoPaper::a(4).into(),
            dpi: 300,
            quality: 95,
            lossless: false,
            margin: CustomSize::zero(),
            force_image_page_fallback_size: false,
            alphabetic_file_sorting: false,
            libreoffice_path: get_default_libre(),
            output_directory: ".".to_owned(),
            output_file: None,
            log: {
                #[cfg(debug_assertions)]
                {
                    LogLevel::Debug
                }
                #[cfg(not(debug_assertions))]
                {
                    LogLevel::Warn
                }
            },
        }
    }
}

// \x1b[0m   -> reset all styles
// \x1b[1m   -> start bold
// \x1b[2m   -> start dim
// \x1b[3m   -> start italic
// \x1b[4m   -> start underline
// \x1b[5m   -> start slow blink
// \x1b[6m   -> start rapid blink (not widely supported)
// \x1b[7m   -> start inverse (swap foreground/background)
// \x1b[8m   -> start hidden (invisible text)
// \x1b[9m   -> start strikethrough

// \x1b[22m -> reset bold/dim
// \x1b[23m -> reset italic
// \x1b[24m -> reset underline
// \x1b[25m -> reset blink
// \x1b[27m -> reset inverse
// \x1b[28m -> reset hidden
// \x1b[29m -> reset strikethrough

pub fn get_command() -> Command {
    let def = Args::default();
    let files = Arg::new("files")
    .required(true)
    .num_args(1..)
        .help("Directories and files to be processed.")
        .long_help(indoc!(r"
            Directories and files to be processed.
            
            Both directory paths and file paths are acceptable.
            If a directory is provided PDFuse will look for applicable file recursively until depth of --recursion-limit is reached.
        "));

    let save_config = Arg::new("save_config")
        .long("save-config")
        .short('s')
        .value_name("FILEPATH")
        .value_hint(ValueHint::FilePath)
        .help("Save input parameters as a TOML configuration file or send to stdout with `-`.");

    let confirm_exit = Arg::new("confirm_exit")
        .long("confirm-exit")
        .action(ArgAction::SetTrue)
        .default_value(def.confirm_exit.to_string())
        .help("Require user input before closing the app.");

    let what_if = Arg::new("what_if")
        .long("whatif")
        .alias("whatif")
        .action(ArgAction::SetTrue)
        .default_value(def.what_if.to_string())
        .help("Dry run: run the program without outputting files.");

    let language = Arg::new("language")
        .short('l')
        .long("language")
        .value_name("IDENTIFIER")
        .help("Specify a language file identifier.");

    let config = Arg::new("config")
        .short('c')
        .long("config")
        .value_name("PATH_TO_CONFIG")
        .value_hint(ValueHint::FilePath)
        .help("Path to a custom configuration file.");

    let recursion_limit = Arg::new("recursion_limit")
        .long("recursion-limit")
        .default_value(def.recursion_limit.to_string())
        .value_parser(clap::value_parser!(usize))
        .help("Recursion depth limit for directories.")
        .long_help("Recursion depth limit for directories.\nLimit of 0 mean only the files in the specified directory are scanned.\nLimit of 1 means the files in any immediate subfolders are also scanned, and so on.");

    let image_page_fallback_size = Arg::new("image_page_fallback_size")
        .short('p')
        .long("image-page-fallback-size")
        .value_name("PAGE_SIZE")
        .default_value(def.image_page_fallback_size.to_string())
        .value_parser(PageSize::try_from_string)
        .help("Fallback page size when adding images.")
        .long_help(indoc!(
            r"
                Fallback page size when adding images.

                Images will usually use the same page size as the preceding document or PDF.
                If nothing precedes an image it will be placed on page size equal to the fallback size.

                Information about how to express a page size can be found in the SIZING section.
            "
        ));

    let dpi = Arg::new("dpi")
        .long("dpi")
        .default_value(def.dpi.to_string())
        .value_parser(clap::value_parser!(u16))
        .help("DPI used when adding images.");

    let quality = Arg::new("quality")
        .long("quality")
        .default_value(def.quality.to_string())
        .value_parser(clap::value_parser!(u8))
        .help("Quality of JPEG image compression.")
        .long_help(indoc!(
            r"
                Quality of JPEG image compression.
                It is expressed as percents - valid range: 1 to 100
            "
        ));

    let lossless = Arg::new("lossless")
        .long("lossless")
        .action(ArgAction::SetTrue)
        .default_value("false")
        .help("Use only lossless compression for images.")
        .long_help(indoc!(
            "
                Use only lossless compression for images.\
                \n
                \x1b[1m\x1b[5mWarning!\x1b[0m This will dramatically increase the size of the output file!
            "
        ));

    let log = Arg::new("log")
        .long("log")
        .default_value(def.log.to_string())
        .ignore_case(true)
        .value_parser(clap::builder::EnumValueParser::<LogLevel>::new())
        .help("Controls which messages are logged into console.");

    let margin = Arg::new("margin")
        .short('m')
        .long("margin")
        .value_name("MARGIN")
        .default_value(def.margin.to_string())
        .value_parser(CustomSize::try_from_string)
        .help("Margin for image pages.");

    let force_image_page_fallback_size = Arg::new("force_image_page_fallback_size")
        .long("force-image-page-fallback-size")
        .action(ArgAction::SetTrue)
        .default_value("false")
        .help("Force the fallback size for image pages, overriding other PDFs.");

    let alphabetic_file_sorting = Arg::new("alphabetic_file_sorting")
        .long("alphabetic-file-sorting")
        .action(ArgAction::SetTrue)
        .default_value(def.alphabetic_file_sorting.to_string())
        .help("Sort paths alphabetically, ignoring input order.")
        .long_help(indoc!(
            r"
                Sort paths alphabetically, ignoring input order.

                By default files will be merged in the same order as they were specified.
                When this flag is enabled, after collecting all files they will be sorted by their paths.
                Useful when drag&dropping items onto the executable (as order in drag&drop may not be obvious).
            "));

    let libreoffice_path = Arg::new("libreoffice_path")
        .long("libreoffice-path")
        .value_name("LIBREOFFICE_PATH")
        .num_args(1..)
        .default_values(def.libreoffice_path)
        .help("Paths to LibreOffice executables for document conversion.")
        .long_help(indoc!(
            r"
                Paths to LibreOffice executables for document conversion.
                
                If the paths are not available, LibreOffice conversion will be disabled.
            "
        ));

    let output_directory = Arg::new("output_directory")
        .short('d')
        .long("output-directory")
        .value_name("OUTPUT_DIRECTORY")
        .value_hint(ValueHint::DirPath)
        .default_value(def.output_directory.to_string())
        .help("Directory for output files.")
        .long_help(indoc!(
            r"
                Directory for output files.

                The output file will have a unique name based on the current date time.
                The exact name of the file also depends on chosen language.
            "
        ));

    let output_file = Arg::new("output_file")
        .short('o')
        .long("output-file")
        .value_name("OUTPUT_FILEPATH")
        .value_hint(ValueHint::FilePath)
        .help("Path for the output file.")
        .long_help(indoc!(
            r"
                Path for the output file.

                The output will try to overwrite any existing file.
            "
        ));

    let group_output = ArgGroup::new("Output")
        .arg(output_file.get_id())
        .arg(output_directory.get_id())
        .multiple({
            #[cfg(test)]
            {
                true
            }
            #[cfg(not(test))]
            {
                false
            }
        });
    Command::new("PDFuse")
        .author("Maciej Krosta")
        .version("1.0")
        .about(
            "Command-line tool to concatenate images, documents and PDFs into a single PDF file.",
        )
        .color(clap::ColorChoice::Auto)
        .arg_required_else_help(true)
        .after_long_help(indoc!(
            r"
            SIZING

            Sizes can be expressed in two ways:
            
             1) With one or two dimensions (width by height) separated by at least one 'x', space or '-'
                 • If only width is specified, height is given the same value.
                 • Supported units (units can be mixed:
                     - Meters m
                     - Millimers mm
                     - Centimeters cm
                     - Inches in
                     - Points pt
                 • Examples:
                     - 5 mm x 7 mm
                     - 5mm x 7mm
                     - 5mmx7mm
                     - 5mm-7mm
                     - 5mm 7mm
                     - 1in
                     - 5mm x 2cm
             2) As an ISO sheet, which will be then translated to standard dimensions.
                 • Supported formats A, B, C
                 • Supported ranks 0-13
                 • Examples: A4, B5, C7
                     - A4 
                     - B5 
                     - C7 
        "
        ))
        .styles(STYLES)
        .arg(files)
        .arg(alphabetic_file_sorting)
        .arg(recursion_limit)
        .next_help_heading("Output")
        .arg(output_file)
        .arg(output_directory)
        .arg(confirm_exit)
        .arg(save_config)
        .arg(what_if)
        .next_help_heading("Imaging")
        .arg(image_page_fallback_size)
        .arg(force_image_page_fallback_size)
        .arg(margin)
        .arg(dpi)
        .arg(quality)
        .arg(lossless)
        .next_help_heading("Miscellaneous")
        .arg(libreoffice_path)
        .arg(language)
        .arg(config)
        .arg(log)
        .group(group_output)
}

macro_rules! set_if_present {
    ($matches:expr,$item:expr,$field:ident, $ty:ty) => {
        if $matches.value_source(stringify!($field)) == Some(clap::parser::ValueSource::CommandLine)
        {
            $item.$field = $matches
                .get_one::<$ty>(stringify!($field))
                .cloned()
                .unwrap();
        }
    };
}
macro_rules! set_if_present_optional {
    ($matches:expr,$item:expr,$field:ident, $ty:ty) => {
        if $matches.value_source(stringify!($field)) == Some(clap::parser::ValueSource::CommandLine)
        {
            if let Some(val) = $matches.get_one::<$ty>(stringify!($field)).cloned() {
                $item.$field = Some(val);
            }
        }
    };
}

pub fn get_args(matches: ArgMatches, base: Option<Args>) -> Args {
    let mut base = base.unwrap_or_default();
    base.files = matches
        .get_many::<String>("files")
        .unwrap()
        .cloned()
        .collect();
    if matches.value_source("libreoffice_path") == Some(clap::parser::ValueSource::CommandLine) {
        base.libreoffice_path = matches
            .get_many::<String>("libreoffice_path")
            .unwrap()
            .cloned()
            .collect();
    }
    set_if_present_optional!(matches, base, save_config, String);
    set_if_present_optional!(matches, base, save_config, String);
    set_if_present_optional!(matches, base, language, String);
    set_if_present_optional!(matches, base, config, String);
    set_if_present_optional!(matches, base, output_file, String);
    set_if_present!(matches, base, output_directory, String);
    set_if_present!(matches, base, recursion_limit, usize);
    set_if_present!(matches, base, image_page_fallback_size, PageSize);
    set_if_present!(matches, base, dpi, u16);
    set_if_present!(matches, base, quality, u8);
    set_if_present!(matches, base, log, LogLevel);
    set_if_present!(matches, base, margin, CustomSize);

    set_if_present!(matches, base, confirm_exit, bool);
    set_if_present!(matches, base, what_if, bool);
    set_if_present!(matches, base, lossless, bool);
    set_if_present!(matches, base, force_image_page_fallback_size, bool);
    set_if_present!(matches, base, alphabetic_file_sorting, bool);
    base
}

#[cfg(test)]
mod tests {
    use super::*;

    fn print_fields(args: &Args) -> Vec<String> {
        let ser = serde_json::to_value(args).unwrap();
        match ser {
            serde_json::Value::Object(map) => map
                .into_iter()
                .map(|x| format!("{}: {}", x.0, x.1.to_string().trim()))
                .collect(),
            _ => panic!(),
        }
    }

    #[test]
    pub fn default_matches_default() {
        let test_file = "file";
        let def = Args {
            files: vec![test_file.to_owned()],
            ..Default::default()
        };
        let cmd = get_args(
            get_command().get_matches_from(vec!["PDFuse", test_file]),
            None,
        );
        assert_eq!(def, cmd);
    }

    #[test]
    pub fn short_long_help_start_match() {
        let cmd = get_command();
        for argument in cmd.get_arguments() {
            if argument.get_long_help().is_none() || argument.get_help().is_none() {
                continue;
            }
            let long_str = argument.get_long_help().unwrap().to_string();
            let start_long = long_str.lines().next().unwrap();
            let short = argument.get_help().map(|x| x.to_string()).unwrap();
            assert_eq!(start_long, short, "{}", argument.get_id());
        }
    }
    #[test]
    pub fn non_default_differs_default() {
        let def = Args {
            alphabetic_file_sorting: !Args::default().alphabetic_file_sorting,
            files: vec!["A".to_owned()],
            save_config: Some("".to_owned()),
            confirm_exit: !Args::default().confirm_exit,
            what_if: !Args::default().what_if,
            language: Some("pl".to_owned()),
            config: Some("c.toml".to_owned()),
            recursion_limit: 1338,
            image_page_fallback_size: IsoPaper::c(5).into(),
            dpi: 420,
            quality: 13,
            lossless: !Args::default().lossless,
            log: LogLevel::Debug,
            margin: IsoPaper::c(4).into(),
            force_image_page_fallback_size: !Args::default().force_image_page_fallback_size,
            libreoffice_path: vec!["none".to_owned()],
            output_directory: "dir".to_owned(),
            output_file: Some("".to_owned()),
        };
        let parsed = get_args(get_command().get_matches_from(vec!["test", "feil"]), None);
        let line1 = print_fields(&def);
        let line2 = print_fields(&parsed);
        let zip = line1.into_iter().zip(line2);
        for (a, b) in zip {
            let a = a.trim();
            let b = b.trim();
            assert_ne!(a, b, "ERROR: {a} - {b}");
        }
    }
    #[test]
    pub fn default_differs_non_default() {
        let def = Args::default();
        let cmd = get_command();
        let arg_strings = vec![
            "pdfuse.exe",
            "file",
            "--alphabetic-file-sorting",
            "--recursion-limit",
            "5",
            "--output-file",
            "output/combined.pdf",
            "--output-directory",
            "output_dir",
            "--confirm-exit",
            "--save-config",
            "config_output.toml",
            "--whatif",
            "--image-page-fallback-size",
            "A10",
            "--force-image-page-fallback-size",
            "--margin",
            "10 mm x 15 mm",
            "--dpi",
            "333",
            "--quality",
            "94",
            "--lossless",
            "--language",
            "de",
            "--config",
            "custom_config.toml",
            "--log",
            "debug",
            "--libreoffice-path",
            "different_path",
        ];
        let matches = get_command().get_matches_from(arg_strings);
        let parsed = get_args(matches.clone(), None);
        for a in cmd.get_arguments() {
            assert!(
                matches.value_source(a.get_id().as_str())
                    == Some(clap::parser::ValueSource::CommandLine),
                "A commandline parameter was not set! {}",
                a.get_id()
            );
        }
        let line1 = print_fields(&def);
        let line2 = print_fields(&parsed);
        let zip = line1.into_iter().zip(line2);
        for (a, b) in zip {
            let a = a.trim();
            let b = b.trim();
            assert_ne!(a, b, "ERROR: {a} - {b}");
        }
    }
}
