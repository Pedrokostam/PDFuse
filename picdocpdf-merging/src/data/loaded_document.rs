use pdfuse_sizing::{CustomSize, Length};
use pdfuse_utils::{create_temp_dir, debug_t, error_t};
use printpdf::lopdf::Document;
use std::{
    fmt::{Debug, Display},
    path::{Path, PathBuf},
    process::Command,
};

use crate::error::{DocumentLoadError, LibreConversionError};

#[derive(Debug)]
pub struct LoadedDocument {
    data: Document,
    source_path: PathBuf,
}
impl From<LoadedDocument> for Document {
    fn from(value: LoadedDocument) -> Self {
        value.data
    }
}
impl Display for LoadedDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Document: {p} pages from \"{path}\"",
            p = self.page_count(),
            path = self.source_path().display()
        )
    }
}
impl LoadedDocument {
    pub fn page_count(&self) -> usize {
        self.data.get_pages().len()
    }
    pub fn source_path(&self) -> &Path {
        &self.source_path
    }
    pub fn page_size(&self) -> Option<CustomSize> {
        let mut page_size: Option<CustomSize> = None;
        for page in self.data.page_iter() {
            let Ok(media_box_array) = self
                .data
                .get_object(page)
                .and_then(|p| p.as_dict())
                .and_then(|d| d.get(b"MediaBox"))
                .and_then(|mb| mb.as_array())
            else {
                debug_t!("debug.invalid_mediabox", document = self);
                continue;
            };
            // all sizes in points
            let x_min = media_box_array[0].as_float().unwrap_or_default();
            let y_min = media_box_array[1].as_float().unwrap_or_default();
            let x_max = media_box_array[2].as_float().unwrap_or_default();
            let y_max = media_box_array[3].as_float().unwrap_or_default();
            let horizontal = Length::from_points(x_max - x_min);
            let vertical = Length::from_points(y_max - y_min);
            if horizontal <= Length::zero() || vertical <= Length::zero() {
                debug_t!("debug.zero_mediabox", document = self);
                continue;
            }
            page_size = Some(CustomSize {
                horizontal,
                vertical,
            });
        }
        if page_size.is_none(){
            error_t!("error.invalid_mediabox",document=self);
        }
        page_size
    }
    pub fn new(
        path: impl AsRef<Path>,
        libre_path: Option<&Path>,
    ) -> Result<LoadedDocument, DocumentLoadError> {
        let ref_path = path.as_ref();
        match ref_path.extension().map(|x| x.to_ascii_lowercase()) {
            Some(ext) if ext == "pdf" => Self::new_pdf_libre(ref_path, libre_path),
            Some(_) => Self::new_libre(ref_path, libre_path).or_else(|_| Self::new_pdf(ref_path)),
            _ => Self::new_pdf_libre(ref_path, libre_path),
        }
    }
    pub fn new_pdf(path: &Path) -> Result<LoadedDocument, DocumentLoadError> {
        Document::load(path)
            .map(|data| LoadedDocument {
                data,
                source_path: path.to_path_buf(),
            })
            .map_err(Into::into)
    }
    pub fn new_libre(
        path: &Path,
        libre_path: Option<&Path>,
    ) -> Result<LoadedDocument, DocumentLoadError> {
        let temp_dir = create_temp_dir();
        let path = convert_document_to_pdf(
            path,
            libre_path.as_ref().expect(
                "At this stage there should be no libre paths if libreoffice is not installed.",
            ),
            &temp_dir,
        )?;
        Self::new_pdf(&path)
    }
    fn new_pdf_libre(
        path: &Path,
        libre_path: Option<&Path>,
    ) -> Result<LoadedDocument, DocumentLoadError> {
        Self::new_pdf(path).or_else(|_| Self::new_libre(path, libre_path))
    }
    fn new_libre_pdf(
        path: &Path,
        libre_path: Option<&Path>,
    ) -> Result<LoadedDocument, DocumentLoadError> {
        Self::new_libre(path, libre_path).or_else(|_| Self::new_pdf(path))
    }
}
pub fn convert_document_to_pdf(
    document_path: &Path,
    libre_exe_path: &Path,
    output_dir: &Path,
) -> Result<PathBuf, LibreConversionError> {
    let extension_path = Path::new(document_path).with_extension("pdf");
    let name = extension_path
        .file_name()
        .expect("Changing extension to pdf shouldn't fail");
    let temp_path = output_dir.join(name);
    let mut cmd = Command::new(libre_exe_path);
    let cmd = cmd
        .arg("--headless")
        .arg("--convert-to")
        .arg("pdf")
        .arg(document_path)
        .arg("--outdir")
        .arg(output_dir);
    let output = cmd.output()?;
    match output.status.success() {
        true => Ok(temp_path),
        false => Err(LibreConversionError::Status(output.status)),
    }
}

