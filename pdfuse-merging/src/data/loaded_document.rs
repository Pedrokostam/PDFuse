use lopdf::{Document, ObjectId};
use pdfuse_parameters::path::SafePath;
use pdfuse_sizing::page::CustomPage;
use pdfuse_sizing::Length;
use pdfuse_utils::debug_t;
use std::{
    fmt::{Debug, Display},
    path::Path,
    process::Command,
};

use crate::{
    conditional_slow_down,
    error::{DocumentLoadError, LibreConversionError, PageSizeParseError},
};

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
#[derive(Clone, Copy, Debug, PartialEq)]
enum PageSizeSearchResult {
    NotFound,
    Size(CustomPage),
    ParentPage(ObjectId),
}
impl PageSizeSearchResult {
    pub fn is_parent(&self) -> bool {
        matches!(self, PageSizeSearchResult::ParentPage(_))
    }
}
impl LoadedDocument {
    pub fn from_document_like(source_path: SafePath, data: Box<Document>) -> Self {
        LoadedDocument { source_path, data }
    }
    pub fn page_count(&self) -> usize {
        self.data.get_pages().len()
    }
    pub fn source_path(&self) -> &SafePath {
        &self.source_path
    }
    fn get_size_or_parent(&self, page: &PageSizeSearchResult) -> PageSizeSearchResult {
        if let PageSizeSearchResult::ParentPage(pg) = page {
            let dict_res = self.data.get_object(*pg).and_then(|p| p.as_dict());
            if let Ok(dict) = dict_res {
                let mediabox_res = dict.get(b"MediaBox").and_then(|m| m.as_array());
                if let Ok(media_box_array) = mediabox_res {
                    let x_min = media_box_array[0].as_float().unwrap_or_default();
                    let y_min = media_box_array[1].as_float().unwrap_or_default();
                    let x_max = media_box_array[2].as_float().unwrap_or_default();
                    let y_max = media_box_array[3].as_float().unwrap_or_default();
                    let horizontal = Length::from_points(x_max - x_min);
                    let vertical = Length::from_points(y_max - y_min);
                    if horizontal <= Length::zero() || vertical <= Length::zero() {
                        debug_t!("debug.zero_mediabox", document = self);
                    }
                    PageSizeSearchResult::Size(CustomPage {
                        horizontal,
                        vertical,
                    })
                } else if let Ok(parent) = dict.get(b"Parent").and_then(|p| p.as_reference()) {
                    PageSizeSearchResult::ParentPage(parent)
                } else {
                    PageSizeSearchResult::NotFound
                }
            } else {
                PageSizeSearchResult::NotFound
            }
        } else {
            *page
        }
    }

    pub fn page_sizes(&self) -> Vec<Result<CustomPage,PageSizeParseError>> {
        self.data
            .page_iter()
            .map(|p| self.page_size_impl(p))
            .collect()
    }

    fn page_size_impl(&self, page: ObjectId) -> Result<CustomPage, PageSizeParseError> {
        let mut search_result = PageSizeSearchResult::ParentPage(page);
        while search_result.is_parent() {
            search_result = self.get_size_or_parent(&search_result);
            // println!("{:?}", search_result);
        }
        match search_result {
            PageSizeSearchResult::Size(custom_page) => {
                // println!("{}", custom_page);
                Ok(custom_page)
            }
            _ => {
                // Parent cannot happen here
                Err(PageSizeParseError {})
            }
        }
    }
    pub fn last_page_size(&self) -> Result<CustomPage,PageSizeParseError> {
        let first_page = self.data.page_iter().last();
        if let Some(fp) = first_page {
            self.page_size_impl(fp)
        } else {
                Err(PageSizeParseError {})
        }
    }
    pub fn first_page_size(&self) -> Result<CustomPage,PageSizeParseError> {
        let first_page = self.data.page_iter().next();
        if let Some(fp) = first_page {
            self.page_size_impl(fp)
        } else {
                Err(PageSizeParseError {})
        }
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
        .as_path()
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
    conditional_slow_down();
    match output.status.success() {
        true => Ok(temp_path),
        false => Err(LibreConversionError::Status(output.status)),
    }
}
