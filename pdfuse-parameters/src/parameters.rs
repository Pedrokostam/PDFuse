
use pdfuse_sizing::{CustomSize, PageSize};
use pdfuse_utils::Indexed;
use crate::{ file_finder,SafePath, SourcePath};

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
    pub libreoffice_path: Option<SafePath>,
    pub output_file: SafePath,
}

/// Returns a unique name based on current time (localized).
fn get_unique_name() -> String {
    let now = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    format!(
        "{stem} {date}.pdf",
        stem = rust_i18n::t!("auto_file_name_stem"),
        date = now
    )
}

// fn get_output_path(args: &Args) -> SafePath {
//     match &args.output_file {
//         Some(path) => path.to_owned(),
//         None => {
//             let unique = get_unique_name();
//             let p = args.output_directory.join(unique);
//             p.into()
//         }
//     }
// }





/// Parameters for operation of the main app, with paths to process.
#[derive(Debug)]
pub struct ParametersWithPaths {
    pub files: Vec<Indexed<SourcePath>>,
    pub parameters: Parameters,
}
unsafe impl Send for ParametersWithPaths {}

impl ParametersWithPaths {
    // pub fn new(args: Args) -> Self {
    //     let libreoffice_path = check_libre(&args.libreoffice_path);
    //     let output_file = get_output_path(&args);
    //     let files = file_finder::get_files(
    //         &args.files,
    //         args.recursion_limit,
    //         libreoffice_path.is_some(),
    //         args.alphabetic_file_sorting,
    //     );
    //     let parameters = Parameters {
    //         confirm_exit: args.confirm_exit,
    //         what_if: args.what_if,
    //         recursion_limit: args.recursion_limit,
    //         image_page_fallback_size: args.image_page_fallback_size,
    //         image_dpi: args.dpi,
    //         image_quality: args.quality,
    //         image_lossless_compression: args.lossless,
    //         margin: args.margin,
    //         force_image_page_fallback_size: args.force_image_page_fallback_size,
    //         alphabetic_file_sorting: args.alphabetic_file_sorting,
    //         libreoffice_path,
    //         output_file,
    //     };
    //     ParametersWithPaths { files, parameters }
    // }
    pub fn deconstruct(self) -> (Vec<Indexed<SourcePath>>, Parameters) {
        (self.files, self.parameters)
    }
}
