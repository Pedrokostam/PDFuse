use super::data::Data;
use crate::error::DocumentLoadError;
use pdfuse_parameters::{
    path::{SafePath, SourcePath},
    Parameters,
};
use pdfuse_utils::Indexed;

pub type PdfRes<T> = std::result::Result<T, DocumentLoadError>;
pub type IndPdfRes<T> = Indexed<PdfRes<T>>;
type PathVec = Vec<Indexed<SafePath>>;
struct Pedefator {
    pdfs: PathVec,
    documents: PathVec,
    sized_images: PathVec,
    unsized_images: PathVec,
}

impl Pedefator {
    pub fn new(files: Vec<Indexed<SourcePath>>, params: &Parameters) -> Self {
        let mut pdfs = vec![];
        let mut documents = vec![];
        let mut sized_images = vec![];
        let mut unsized_images = vec![];
        for file in files {
            let index = file.index();
            match file.take_out() {
                SourcePath::Pdf(safe_path) => pdfs.push(Indexed::new(index, safe_path)),
                SourcePath::Image(safe_path) => todo!(),
                SourcePath::LibreDocument(safe_path) => todo!(),
            }
        }
        Pedefator {
            pdfs: (),
            documents: (),
            sized_images: (),
            unsized_images: (),
        }
    }
}
