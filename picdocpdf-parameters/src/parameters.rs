use std::path::PathBuf;

use pdfuse_sizing::{CustomSize, PageSize};
use pdfuse_utils::Indexed;
use serde::{Deserialize, Serialize};

use crate::{errors::ConfigError, Args, SourcePath};

#[derive(Debug,Clone)]
pub struct Parameters{
    pub confirm_exit: bool,
    pub quiet: bool,
    pub what_if: bool,
    pub recursion_limit: usize,
    pub image_page_fallback_size: PageSize,
    pub dpi: u16,
    pub margin: CustomSize,
    pub force_image_page_fallback_size: bool,
    pub alphabetic_file_sorting: bool,
    pub libreoffice_path: Option<PathBuf>,
    pub output_file: String,
}
#[derive(Debug)]
pub struct ParametersWithPaths{
    pub files:Vec<Indexed<SourcePath>>,
    pub parameters:Parameters
}
unsafe impl Send for ParametersWithPaths{}
impl ParametersWithPaths{
    pub fn parse()->Result<Self,ConfigError>{
        let a = Args::create()?;
        a.make_parameters().map_err(|ivnalid| ivnalid.into())
    }
}