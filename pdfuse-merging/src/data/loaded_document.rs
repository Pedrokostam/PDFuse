use pdfuse_parameters::SafePath;
use pdfuse_sizing::{CustomSize, Length};
use pdfuse_utils::{debug_t, error_t};
use lopdf::Document;
use std::{
    fmt::{Debug, Display},
    path::{Path},
    process::Command,
};

use crate::error::{DocumentLoadError, LibreConversionError};

#[derive(Debug)]
pub struct LoadedDocument {
    data: Box<Document>,
    source_path: SafePath,
}
impl From<LoadedDocument> for Document {
    fn from(value: LoadedDocument) -> Self {
        *value.data
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
    pub fn load_pdf(path: &Path) -> Result<LoadedDocument, DocumentLoadError> {
        Document::load(path)
            .map(|data| LoadedDocument {
                data: Box::new(data),
                source_path: SafePath::new(path),
            })
            .map_err(Into::into)
    }
}
pub fn convert_document_to_pdf(
    document_path: &SafePath,
    libre_exe_path: &SafePath,
    output_dir: &SafePath,
) -> Result<SafePath, LibreConversionError> {
    let extension_path = document_path.with_extension("pdf");
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

