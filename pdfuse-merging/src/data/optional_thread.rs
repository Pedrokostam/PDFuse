use std::path::PathBuf;

use pdfuse_parameters::Parameters;
use pdfuse_utils::{create_temp_dir, Indexed};

use crate::DocumentLoadError;

use super::{loaded_document, preload_pdf, Data, PdfResult};

pub enum OptionalThread {
    NoOp,
    Thread(std::thread::JoinHandle<Vec<Indexed<Result<Data, DocumentLoadError>>>>),
}

impl OptionalThread {
    pub fn create(document_paths: Vec<Indexed<PathBuf>>, parameters: &Parameters) -> Self {
        if document_paths.is_empty() {
            return Self::NoOp;
        }
        let parameters = parameters.clone();
        let handle = std::thread::spawn(move || {
            let paths = convert_data_to_documents(document_paths, &parameters);
            let output: Vec<Indexed<PdfResult<Data>>> = paths
                .into_iter()
                .map(|indexed| {
                    indexed.map_with_index(|result| match result {
                        Ok(path) => preload_pdf(path),
                        Err(err) => Err(err),
                    })
                })
                .collect();
            output
        });
        Self::Thread(handle)
    }
    pub fn get_converted_data(self) -> Vec<Indexed<Result<Data, DocumentLoadError>>> {
        match self {
            OptionalThread::NoOp => vec![],
            OptionalThread::Thread(join_handle) => join_handle
                .join()
                .expect("The Libre thread should not fail"),
        }
    }
}

fn convert_data_to_documents(
    document_paths: Vec<Indexed<PathBuf>>,
    parameters: &Parameters,
) -> Vec<Indexed<PdfResult<PathBuf>>> {
    if parameters.libreoffice_path.is_none() {
        return vec![];
    }
    let temp_dir = create_temp_dir();
    let libre_path = parameters.libreoffice_path.clone().unwrap();
    document_paths
        .into_iter()
        .map(|p| {
            p.map_with_index(|x| {
                loaded_document::convert_document_to_pdf(&x, &libre_path, &temp_dir)
                    .map_err(Into::into)
            })
        })
        .collect()
}
