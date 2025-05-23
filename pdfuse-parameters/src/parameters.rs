use std::{path::PathBuf, str::FromStr};

use pdfuse_sizing::{CustomSize, PageSize};
use pdfuse_utils::Indexed;

use crate::{errors::ConfigError, paths, Args, SourcePath};

/// Parameters used during conversion, creation, and merging of PDFs.
#[derive(Debug, Clone, Default)]
pub struct Parameters {
    pub confirm_exit: bool,
    pub what_if: bool,
    pub recursion_limit: usize,
    pub image_page_fallback_size: PageSize,
    pub image_dpi: u16,
    pub image_quality: u8,
    pub image_lossless_compression: bool,
    pub margin: CustomSize,
    pub force_image_page_fallback_size: bool,
    pub alphabetic_file_sorting: bool,
    pub libreoffice_path: Option<PathBuf>,
    pub output_file: PathBuf,
}

impl Parameters {
    pub fn from_args(args: Args) -> Parameters {
        let libreoffice_path = check_libre(&args.libreoffice_path);
        Parameters {
            confirm_exit: args.confirm_exit,
            what_if: args.what_if,
            recursion_limit: args.recursion_limit,
            image_page_fallback_size: args.image_page_fallback_size,
            image_dpi: args.dpi,
            image_quality: args.quality,
            image_lossless_compression: args.lossless,
            margin: args.margin,
            force_image_page_fallback_size: args.force_image_page_fallback_size,
            alphabetic_file_sorting: args.alphabetic_file_sorting,
            libreoffice_path,
            output_file: args.output_file,
        }
    }
}

fn check_libre(paths: &[String]) -> Option<PathBuf> {
    for libre_path in paths {
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

/// Parameters for operation of the main app, with paths to process.
#[derive(Debug)]
pub struct ParametersWithPaths {
    pub files: Vec<Indexed<SourcePath>>,
    pub parameters: Parameters,
}
unsafe impl Send for ParametersWithPaths {}

impl ParametersWithPaths {
    pub fn parse() -> Result<Self, ConfigError> {
        let a = Args::create()?;
        a.make_parameters()
    }
}
