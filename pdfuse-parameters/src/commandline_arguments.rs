use crate::{
    errors::ConfigError,
    file_finder,
    parameters::{Parameters, ParametersWithPaths},
    paths::{self, expand_path},
    SourcePath,
};
use clap::{
    builder::styling, ArgAction, ColorChoice, CommandFactory, FromArgMatches, Parser, ValueEnum,
    ValueHint,
};
use pdfuse_sizing::{CustomSize, IsoPaper, PageSize};
use pdfuse_utils::Indexed;
use rust_i18n::t;
use serde::{ Deserialize, Serialize};
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

/// Use the value for the field from the default implementation
macro_rules! def {
    ($field:ident) => {
        Args::default().$field
    };
}
#[derive(Debug, Clone, Copy, ValueEnum, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogLevel {
    /// A level lower than all log levels.
    Off,
    /// Corresponds to the `Error` log level.
    Error,
    /// Corresponds to the `Warn` log level.
    Warn,
    /// Corresponds to the `Info` log level.
    Info,
    /// Corresponds to the `Debug` log level.
    Debug,
    /// Corresponds to the `Trace` log level.
    Trace,
}

impl From<LogLevel> for log::LevelFilter {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Off => log::LevelFilter::Off,
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Trace => log::LevelFilter::Trace,
        }
    }
}

/// Command-line tool to process directories and files with various options.
///
/// Directories will be searched recursively looking for images, PDFs, and office document formats.
/// Relative paths are based on the current working directory.
///
/// Can be serialized to save a configuration.
#[derive(Parser, Debug, Serialize, Deserialize, PartialEq)]
#[command(author, version, about, color=ColorChoice::Auto,styles=STYLES,arg_required_else_help = true)]
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

    /// Require user input before closing the app.
    ///
    /// Useful when starting the app from desktop (allows the user to read the logs).
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
    #[arg(short = 'p', long,  value_name = "PAGE_SIZE", value_parser =PageSize::try_from_string,default_value_t)]
    pub image_page_fallback_size: PageSize,

    /// DPI used when adding images.
    #[arg(long, default_value_t = def!(dpi))]
    pub dpi: u16,

    /// Quality of JPEG image compression.
    #[arg(long, default_value_t = def!(quality))]
    pub quality: u8,

    /// Use only lossless compression for images.
    #[arg(long, action = ArgAction::SetTrue, default_value_t = def!(lossless))]
    pub lossless: bool,

    #[arg(long,value_enum, default_value_t = def!(log))]
    pub log: LogLevel,

    /// Margin for image pages.
    #[arg(short = 'm', long, value_name = "MARGIN",value_parser =CustomSize::try_from_string,default_value_t= def!(margin))]
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

impl Args {
    pub fn is_valid(&self) -> bool {
        !self.files.is_empty()
    }

    pub fn make_parameters(self) -> Result<ParametersWithPaths, ConfigError> {
        self.save_config()?;
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
            image_dpi: self.dpi,
            image_quality: self.quality,
            image_lossless_compression: self.lossless,
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

    /// Saves this instance of [`Args`] to the path specified in the [`save_config`] field. Creates all needed directories.
    ///
    /// # Errors
    ///
    /// This function will return an error if toml cannot be converted to string, or the current working directory cannot be found (never?).
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
    ) -> Result<Vec<Indexed<SourcePath>>, ConfigError> {
        let q = file_finder::get_files(&self.files, max_depth, allow_office_docs, sort);
        match q {
            v if !v.is_empty() => Ok(v),
            _ => Err(ConfigError::NoValidFiles),
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

    /// Creates new Args from the parameters given to the app (when items are None) or from the given items.
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    fn create_from<I, T>(items: Option<I>) -> Result<Self, ConfigError>
    where
        I: IntoIterator<Item = T> + Debug,
        T: Into<OsString> + Clone,
    {
        let matches = match items {
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
        let is_default_config = args.config.is_none();
        let config_path_property = &args.config.as_deref().unwrap_or(DEFAULT_CONFIG_PATH);
        // Get expanded path
        let expanded_config_path = expand_path(config_path_property)
            .ok_or_else(|| ConfigError::MalformedPath(config_path_property.to_string()))?;
        let config_path = Path::new(&expanded_config_path);
        if config_path.exists() {
            // Try to read the config file. Exit on fail.
            let loaded_config_text = fs::read_to_string(config_path)?;
            let loaded = toml::from_str::<Args>(&loaded_config_text)?;

            hack!(mut args, loaded, confirm_exit, matches); //: false,
            hack!(mut args, loaded, quiet, matches); //: false,
            hack!(mut args, loaded, what_if, matches); //: false,
            hack!(mut args, loaded, language, matches); //: None,
            hack!(mut args, loaded, recursion_limit, matches); //: 4,
            hack!(mut args, loaded, image_page_fallback_size, matches); //: IsoPaper::a(4).into(),
            hack!(mut args, loaded, dpi, matches); //: 300,
            hack!(mut args, loaded, margin, matches); //: CustomSize::zero(),
            hack!(mut args, loaded, force_image_page_fallback_size, matches); //: false,
            hack!(mut args, loaded, log, matches); //: depends,
            hack!(mut args, loaded, quality, matches); //: 95,
            hack!(mut args, loaded, lossless, matches); //: false,
            hack!(mut args, loaded, alphabetic_file_sorting, matches); //: false,
            hack!(mut args, loaded, libreoffice_path, matches); //: get_default_libre(),
            hack!(mut args, loaded, output_directory, matches); //: ".".to_owned(),
        } else if !is_default_config {
            // config does not exist and it is not default
            Err(ConfigError::MissingConfigError(
                expanded_config_path.to_owned(),
            ))?
        }
        log::set_max_level(args.log.into());
        Ok(args)
    }

    fn load_language(&self) {
        if let Some(lang) = self.language.as_ref() {
            rust_i18n::set_locale(lang);
        }
    }
}

/// Returns a unique name based on current time (localized).
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
                    LogLevel::Trace
                }

                #[cfg(not(debug_assertions))]
                {
                    LogLevel::Error
                }
            },
        }
    }
}

#[cfg(test)]
mod tests;
