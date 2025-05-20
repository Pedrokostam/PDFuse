use std::path::PathBuf;

use pdfuse_sizing::{CustomSize, PageSize};
use pdfuse_utils::Indexed;

use crate::{errors::ConfigError, Args, SourcePath};

/// Parameters used during conversion, creation, and merging of PDFs.
#[derive(Debug,Clone,Default)]
pub struct Parameters{
    pub confirm_exit: bool,
    pub quiet: bool,
    pub what_if: bool,
    pub recursion_limit: usize,
    pub image_page_fallback_size: PageSize,
    pub image_dpi: u16,
    pub image_quality:u8,
    pub image_lossless_compression:bool,
    pub margin: CustomSize,
    pub force_image_page_fallback_size: bool,
    pub alphabetic_file_sorting: bool,
    pub libreoffice_path: Option<PathBuf>,
    pub output_file: String,
}

/// Parameters for operation of the main app, with paths to process.
#[derive(Debug)]
pub struct ParametersWithPaths{
    pub files:Vec<Indexed<SourcePath>>,
    pub parameters:Parameters
}
unsafe impl Send for ParametersWithPaths{}

impl ParametersWithPaths{
    pub fn parse()->Result<Self,ConfigError>{
        let a = Args::create()?;
        a.make_parameters()
    }
}