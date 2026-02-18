use crate::{
    path::{SafePath, SourcePath},
    Bookmarks,
};

use pdfuse_sizing::paper::{CustomPage, Page};
use pdfuse_utils::Indexed;

/// Parameters used during conversion, creation, and merging of PDFs.
#[derive(Debug, Clone, Default)]
pub struct Parameters {
    pub confirm_exit: bool,
    pub what_if: bool,
    pub recursion_limit: usize,
    pub image_page_fallback_size: Page,
    pub image_dpi: u16,
    pub image_quality: u8,
    pub image_lossless_compression: bool,
    pub margin: CustomPage,
    pub force_image_page_fallback_size: bool,
    pub alphabetic_file_sorting: bool,
    pub bookmarks: Bookmarks,
    pub libreoffice_path: Option<SafePath>,
    pub output_file: SafePath,
}

/// Parameters for operation of the main app, with paths to process.
#[derive(Debug)]
pub struct ParametersWithPaths {
    pub files: Vec<Indexed<SourcePath>>,
    pub parameters: Parameters,
}
unsafe impl Send for ParametersWithPaths {}

impl ParametersWithPaths {
    pub fn deconstruct(self) -> (Vec<Indexed<SourcePath>>, Parameters) {
        (self.files, self.parameters)
    }
}
