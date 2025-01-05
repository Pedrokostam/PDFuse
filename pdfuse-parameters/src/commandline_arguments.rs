use crate::{
    errors::{ConfigError, InvalidInputFileError},
    file_finder,
    parameters::{Parameters, ParametersWithPaths},
    paths::{self, expand_path},
    SourcePath,
};
use clap::{
    builder::styling, ArgAction, ColorChoice, CommandFactory, FromArgMatches, Parser, ValueHint,
};
use pdfuse_sizing::{CustomSize, IsoPaper, LengthParseError, PageSize, PageSizeError};
use pdfuse_utils::Indexed;
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use std::{
    env,
    ffi::OsString,
    fmt::Debug,
    fs, 
    path::{Path, PathBuf},
    str::FromStr,
};

const DEFAULT_CONFIG_PATH: &str = "config_auto.toml";
const STYLES: styling::Styles = styling::Styles::styled()
    .header(styling::AnsiColor::Green.on_default().bold())
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
/// Replaces the `$field` in `$parsed` with the value from `$loaded` if `$field` was not specified in the commandline.
macro_rules! hack {
    (mut $parsed:expr,$loaded:expr,$field:ident,$matches:expr) => {{
        let source = $matches
            .value_source(stringify!($field))
            .unwrap_or(clap::parser::ValueSource::DefaultValue);
        let present = source == clap::parser::ValueSource::CommandLine;
        if (!present) {
            $parsed.$field = $loaded.$field.clone()
        }
    }};
}
macro_rules! def {
    ($field:ident) => {
        Args::default().$field
    };
}
/// Command-line tool to process directories and files with various options.
///
/// Directories will be searched recursively looking for images, PDFs, and office document formats.
/// Relative paths are based on the current working directory.
#[derive(Parser, Debug, Serialize, Deserialize)]
#[command(author, version, about, color=ColorChoice::Always,styles=STYLES,arg_required_else_help = true)]
#[serde(default)]
pub struct Args {
    /// Directories and files to be processed.
    /// Directories will be searched recursively for images, PDFs, and office documents.
    #[arg(required = true)]
    #[serde(skip_serializing)]
    pub files: Vec<String>,

    /// Save input parameters as a TOML configuration file or send to stdout with `-`.
    #[arg(short, long, value_name = "FILEPATH", value_hint = ValueHint::FilePath)]
    #[serde(skip_serializing)]
    pub save_config: Option<String>,

    /// Confirm exit.
    #[arg(long, action = ArgAction::SetTrue, default_value_t = def!(confirm_exit))]
    pub confirm_exit: bool,

    /// Quiet mode: suppress all output.
    #[arg(short, long, action = ArgAction::SetTrue, default_value_t = def!(quiet))]
    pub quiet: bool,

    /// Dry run: run the program without outputting files.
    #[arg(long, alias = "whatif", action = ArgAction::SetTrue, default_value_t = def!(what_if))]
    pub what_if: bool,

    /// Specify a language file identifier.
    #[arg(short, long, value_name = "IDENTIFIER")]
    pub language: Option<String>,

    /// Path to a custom configuration file.
    #[arg(short, long, value_name = "PATH_TO_CONFIG", value_hint = ValueHint::FilePath)]
    #[serde(skip_serializing)]
    pub config: Option<String>,

    /// Recursion depth limit for directories.
    #[arg(long, default_value_t = def!(recursion_limit))]
    pub recursion_limit: usize,

    /// Fallback page size when adding images.
    #[arg(short = 'p', long,  value_name = "PAGE_SIZE", value_parser =parse_page_size,default_value_t)]
    pub image_page_fallback_size: PageSize,

    /// DPI used when adding images.
    #[arg(long, default_value_t = def!(dpi))]
    pub dpi: u16,

    /// Margin for image pages.
    #[arg(short = 'm', long, value_name = "MARGIN",value_parser =parse_custom_size,default_value_t= def!(margin))]
    pub margin: CustomSize,

    /// Force the fallback size for image pages, overriding other PDFs.
    #[arg(long, action = ArgAction::SetTrue, default_value_t = def!(force_image_page_fallback_size))]
    pub force_image_page_fallback_size: bool,

    /// Sort paths alphabetically, ignoring input order.
    #[arg(long,alias="afs", action = ArgAction::SetTrue, default_value_t = def!(alphabetic_file_sorting))]
    pub alphabetic_file_sorting: bool,

    /// Paths to LibreOffice executables for document conversion.
    #[arg(long, value_name = "LIBREOFFICE_PATH", num_args = 1.., default_values_t = def!(libreoffice_path))]
    pub libreoffice_path: Vec<String>,

    /// Directory for output files.
    #[arg(short = 'd', long, value_name = "OUTPUT_DIRECTORY", value_hint = ValueHint::DirPath,default_value_t = def!(output_directory))]
    pub output_directory: String,

    /// Path for the output file.
    #[arg(short = 'o', long, value_name = "OUTPUT_FILEPATH", value_hint = ValueHint::FilePath)]
    #[serde(skip_serializing)]
    pub output_file: Option<String>,
}

fn parse_page_size(arg: &str) -> Result<PageSize, PageSizeError> {
    PageSize::try_from_string(arg)
}
fn parse_custom_size(arg: &str) -> Result<CustomSize, LengthParseError> {
    CustomSize::try_from_string(arg)
}
impl Args {
    pub fn is_valid(&self) -> bool {
        !self.files.is_empty()
    }
    pub fn make_parameters(mut self) -> Result<ParametersWithPaths, InvalidInputFileError> {
        self.save_config();
        let libreoffice_path = self.check_libre();
        let office_good = libreoffice_path.is_some();
        let files = self.get_flat_files(
            self.recursion_limit,
            office_good,
            self.alphabetic_file_sorting,
        )?;
        let parameters = Parameters {
            libreoffice_path,
            alphabetic_file_sorting: self.alphabetic_file_sorting,
            confirm_exit: self.confirm_exit,
            dpi: self.dpi,
            force_image_page_fallback_size: self.force_image_page_fallback_size,
            image_page_fallback_size: self.image_page_fallback_size,
            margin: self.margin,
            quiet: self.quiet,
            what_if: self.what_if,
            recursion_limit: self.recursion_limit,
            output_file: self.get_output_path(),
        };
        Ok(ParametersWithPaths { files, parameters })
    }
    fn check_libre(&self) -> Option<PathBuf> {
        for libre_path in &self.libreoffice_path {
            let expanded_path = paths::expand_path(libre_path);
            let Some(expanded_path_un) = expanded_path else {
                continue;
            };
            if paths::is_executable(&expanded_path_un) {
                return Some(
                    PathBuf::from_str(&expanded_path_un)
                        .expect("Path was already checked, should not fail"),
                );
            }
        }
        None
    }
    fn save_config(&self) -> Result<(), ConfigError> {
        let Some(save_config_path) = &self.save_config else {
            return Ok(());
        };
        let canon_path = fs::canonicalize(save_config_path)?;
        let content = toml::to_string_pretty(self)?;
        fs::create_dir_all(canon_path.parent().unwrap_or(env::current_dir()?.as_path()))?;
        fs::write(canon_path, content)?;
        Ok(())
    }

    fn get_flat_files(
        &self,
        max_depth: usize,
        allow_office_docs: bool,
        sort: bool,
    ) -> Result<Vec<Indexed<SourcePath>>, InvalidInputFileError> {
        let q = file_finder::get_files(&self.files, max_depth, allow_office_docs, sort);
        match q {
            v if !v.is_empty() => Ok(v),
            _ => Err(InvalidInputFileError {}),
        }
    }

    fn get_output_path(&self) -> String {
        match self
            .output_file
            .as_ref()
            .and_then(|fp| paths::expand_path(fp))
        {
            Some(path) => path,
            None => {
                let unique = get_unique_name();
                let mut output = Path::new(&self.output_directory).to_path_buf();
                output.push(unique);
                let with_file = output.as_path().to_string_lossy().into_owned();
                paths::expand_path(&with_file).expect("Expected a valid path for target directory")
            }
        }
    }
    pub fn create() -> Result<Args, ConfigError> {
        Self::create_from::<Vec<OsString>, OsString>(None)
    }

    pub fn load_config_file(path: &str) -> Result<Args, ConfigError> {
        let contents = fs::read_to_string(path)?;
        Ok(toml::from_str::<Args>(&contents)?)
    }
    fn create_from<I, T>(items: Option<I>) -> Result<Self, ConfigError>
    where
        I: IntoIterator<Item = T> + Debug,
        T: Into<OsString> + Clone,
    {
        let itera = format!("{:?}", &items);
        let mut matches = match items {
            Some(x) => <Self as CommandFactory>::command().get_matches_from(x),
            None => <Self as CommandFactory>::command().get_matches(),
        };
        let args = Self::parse_and_load(matches)?;
        args.save_config()?;
        args.load_language();
        Ok(args)
    }

    fn parse_and_load(matches: clap::ArgMatches) -> Result<Self, ConfigError> {
        let parse_result = <Self as FromArgMatches>::from_arg_matches(&matches)
            .map_err(|e| e.format(&mut Self::command()));
        let mut args = match parse_result {
            Ok(s) => s,
            Err(e) => {
                // Since this is more of a development-time error, we aren't doing as fancy of a quit
                // as `get_matches`
                e.exit();
            }
        };
        let expanded_config_path_opt = expand_path(
            &args
                .config
                .as_ref()
                .unwrap_or(&DEFAULT_CONFIG_PATH.to_owned())
                .to_owned(),
        );
        // res.save_config=Some("config.toml".to_owned());
        // res.save_config();
        let config_path = match &expanded_config_path_opt {
            Some(s) => Path::new(s),
            None => return Ok(args),
        };
        if !config_path.exists() {
            return Ok(args);
        }
        let Some(loaded) = fs::read_to_string(config_path)
            .ok()
            .and_then(|s| toml::from_str::<Args>(&s).ok())
        else {
            return Ok(args);
        };
        hack!(mut args, loaded, confirm_exit, matches); //: false,
        hack!(mut args, loaded, quiet, matches); //: false,
        hack!(mut args, loaded, what_if, matches); //: false,
        hack!(mut args, loaded, language, matches); //: None,
        hack!(mut args, loaded, recursion_limit, matches); //: 4,
        hack!(mut args, loaded, image_page_fallback_size, matches); //: IsoPaper::a(4).into(),
        hack!(mut args, loaded, dpi, matches); //: 300,
        hack!(mut args, loaded, margin, matches); //: CustomSize::zero(),
        hack!(mut args, loaded, force_image_page_fallback_size, matches); //: false,
        hack!(mut args, loaded, alphabetic_file_sorting, matches); //: false,
        hack!(mut args, loaded, libreoffice_path, matches); //: get_default_libre(),
        hack!(mut args, loaded, output_directory, matches); //: ".".to_owned(),
        Ok(args)
    }

    fn load_language(&self) {
        if let Some(lang) = self.language.as_ref() {
            rust_i18n::set_locale(lang);
        }
    }
}

fn get_unique_name() -> String {
    let now = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    format!(
        "{stem} {date}.pdf",
        stem = t!("auto_file_name_stem"),
        date = now
    )
}
impl Default for Args {
    fn default() -> Self {
        Self {
            files: Default::default(),
            save_config: None,
            confirm_exit: false,
            quiet: false,
            what_if: false,
            language: None,
            config: None,
            recursion_limit: 4,
            image_page_fallback_size: IsoPaper::a(4).into(),
            dpi: 300,
            margin: CustomSize::zero(),
            force_image_page_fallback_size: false,
            alphabetic_file_sorting: false,
            libreoffice_path: get_default_libre(),
            output_directory: ".".to_owned(),
            output_file: None,
        }
    }
}
#[cfg(test)]
mod tests;
