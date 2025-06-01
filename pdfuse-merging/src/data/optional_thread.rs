use std::thread::JoinHandle;

use indicatif::{MultiProgress, ProgressBar, ProgressIterator};
use pdfuse_parameters::{create_temp_dir, Parameters, SafePath};
use pdfuse_utils::{
    get_registered_busy_indicator, get_registered_progress_iterator, get_registered_progress_iterator_parallel, log::debug, Indexed
};

use crate::DocumentLoadError;

use super::{loaded_document, preload_pdf, Data, PdfResult};

pub struct ProgressJoinHandle {
    handle: JoinHandle<Vec<Indexed<Result<Data, DocumentLoadError>>>>,
}

impl ProgressJoinHandle {
    pub fn new(document_paths: Vec<Indexed<SafePath>>, parameters: &Parameters) -> Self {
        let parameters = parameters.clone();

        let handle = std::thread::spawn(move || {
            let paths = convert_data_to_documents(document_paths, &parameters);
            let output: Vec<Indexed<PdfResult<Data>>> = paths
                .into_iter()
                .map(|indexed| {
                    indexed.map_with_index(|result: Result<SafePath, DocumentLoadError>| {
                        match result {
                            Ok(path) => preload_pdf(path),
                            Err(err) => Err(err),
                        }
                    })
                })
                .collect();
            output
        });
        debug!("_&doc thread started");
        ProgressJoinHandle { handle }
    }

    pub fn join(self) -> Result<IndexedResults, Box<dyn std::any::Any + Send>> {
        self.handle.join()
    }
}
type IndexedResults = Vec<Indexed<Result<Data, DocumentLoadError>>>;

pub enum OptionalThread {
    NoOp,
    Thread(ProgressJoinHandle),
}

impl OptionalThread {
    pub fn create(document_paths: Vec<Indexed<SafePath>>, parameters: &Parameters) -> Self {
        if document_paths.is_empty() {
            return Self::NoOp;
        }
        Self::Thread(ProgressJoinHandle::new(document_paths, parameters))
    }

    pub fn get_converted_data(self) -> Vec<Indexed<Result<Data, DocumentLoadError>>> {
        debug!("_&Waiting for doc thread to finish");
        match self {
            OptionalThread::NoOp => vec![],
            OptionalThread::Thread(join_handle) => join_handle
                .join()
                .expect("The Libre thread should not fail"),
        }
    }
}

fn convert_data_to_documents(
    document_paths: Vec<Indexed<SafePath>>,
    parameters: &Parameters,
) -> Vec<Indexed<PdfResult<SafePath>>> {
    if parameters.libreoffice_path.is_none() {
        return vec![];
    }
    let temp_dir = create_temp_dir();
    let libre_path = parameters.libreoffice_path.clone().unwrap();
    get_registered_progress_iterator(
        document_paths.into_iter(),
        "_&Converting documents to PDF...",
    )
    .map(|p| {
        p.map_with_index(|x| {
            loaded_document::convert_document_to_pdf(&x, &libre_path, &temp_dir).map_err(Into::into)
        })
    })
    .collect()
}
