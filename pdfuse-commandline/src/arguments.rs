use std::{default, fs};

use pdfuse_parameters::{
    get_files, Bookmarks, ConfigError, LogLevel, Parameters, ParametersWithPaths, SafeDestination, SafePath
};
use pdfuse_sizing::{CustomSize, IsoPaper, PageSize};
use pdfuse_utils::debug_t;
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Args {
    #[cfg_attr(not(test), serde(skip))]
    pub files: Vec<SafePath>,
    #[cfg_attr(not(test), serde(skip))]
    pub save_config: Option<SafeDestination>,
    pub confirm_exit: bool,
    #[cfg_attr(not(test), serde(skip))]
    pub what_if: bool,
    pub language: Option<String>,
    #[cfg_attr(not(test), serde(skip))]
    pub config: Option<SafePath>,
    pub recursion_limit: usize,
    pub image_page_fallback_size: PageSize,
    pub dpi: u16,
    pub quality: u8,
    pub lossless: bool,
    pub log: LogLevel,
    pub margin: CustomSize,
    pub force_image_page_fallback_size: bool,
    pub alphabetic_file_sorting: bool,
    pub bookmarks:Bookmarks,
    pub libreoffice_path: Vec<SafePath>,
    pub output_directory: SafePath,
    #[cfg_attr(not(test), serde(skip))]
    pub output_file: Option<SafePath>,
}
pub(crate) const DEFAULT_LIBRE_PATHS: &[&str] = {
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

pub(crate) fn get_default_libre() -> Vec<SafePath> {
    DEFAULT_LIBRE_PATHS
        .iter()
        .map(|p| SafePath::from(*p))
        .collect()
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
            bookmarks:Default::default(),
            libreoffice_path: get_default_libre(),
            output_directory: ".".into(),
            output_file: None,
            log: if cfg!(debug_assertions) {
                LogLevel::Debug
            } else {
                LogLevel::Warn
            },
        }
    }
}

pub(crate) fn check_libre(paths: &[SafePath]) -> Option<SafePath> {
    for libre_path in paths {
        if libre_path.is_executable() {
            return Some(libre_path.to_owned());
        }
    }
    None
}
fn get_unique_name() -> String {
    let now = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    format!(
        "{stem} {date}.pdf",
        stem = rust_i18n::t!("auto_file_name_stem"),
        date = now
    )
}

fn get_output_path(args: &Args) -> SafePath {
    match &args.output_file {
        Some(path) => path.to_owned(),
        None => {
            let unique = get_unique_name();
            args.output_directory.join(unique)
        }
    }
}
impl Args {
    pub fn from_toml_file(path: &SafePath) -> Result<Args, ConfigError> {
        debug_t!("debug.reading_preset", path = path);
        if !path.exists() {
            Err(ConfigError::MissingConfigError(path.clone()))?
        }
        let contents = fs::read_to_string(path)?;
        Ok(toml::from_str::<Args>(&contents)?)
    }
    pub fn to_parameters(self) -> ParametersWithPaths {
        let libreoffice_path = check_libre(&self.libreoffice_path);
        let output_file = get_output_path(&self);
        let files = get_files(
            &self.files,
            self.recursion_limit,
            libreoffice_path.is_some(),
            self.alphabetic_file_sorting,
        );
        let parameters = Parameters {
            confirm_exit: self.confirm_exit,
            what_if: self.what_if,
            recursion_limit: self.recursion_limit,
            image_page_fallback_size: self.image_page_fallback_size,
            image_dpi: self.dpi,
            image_quality: self.quality,
            image_lossless_compression: self.lossless,
            margin: self.margin,
            force_image_page_fallback_size: self.force_image_page_fallback_size,
            alphabetic_file_sorting: self.alphabetic_file_sorting,
            bookmarks:self.bookmarks,
            libreoffice_path,
            output_file,
        };
        ParametersWithPaths { files, parameters }
    }

    /// Saves config, only if the argument --save-config was specified
    pub fn save_config(&self) -> Result<(), ConfigError> {
        if let Some(destination) = &self.save_config {
            let toml_string = toml::to_string(self)?;
            destination.write_to(&toml_string)?
        }
        Ok(())
    }

    pub fn set_localization(&self) {
        if let Some(id) = self.language.as_ref() {
            pdfuse_utils::set_localization(id);
        }
    }
}
