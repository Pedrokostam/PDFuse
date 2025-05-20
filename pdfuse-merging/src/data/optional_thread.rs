use std::{path::PathBuf, thread::JoinHandle};

use indicatif::{MultiProgress, ProgressBar};
use pdfuse_parameters::Parameters;
use pdfuse_utils::{create_temp_dir, get_progress_indicator, Indexed};

use crate::DocumentLoadError;

use super::{loaded_document, preload_pdf, Data, PdfResult};

pub struct ProgressJoinHandle {
    handle: JoinHandle<Vec<Indexed<Result<Data, DocumentLoadError>>>>,
}

impl ProgressJoinHandle {
    pub fn new(
        document_paths: Vec<Indexed<PathBuf>>,
        parameters: &Parameters,
        parent_bar: MultiProgress,
    ) -> Self {
        let parameters = parameters.clone();
        let progress_bar = get_progress_indicator(
            document_paths.len() as u64,
            "Converting documents to PDF...",
        );
        parent_bar.add(progress_bar.clone());
        let handle = std::thread::spawn(move || {
            let paths = convert_data_to_documents(document_paths, &parameters, progress_bar);
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

        ProgressJoinHandle { handle }
    }

    
    pub fn join(
        self,
    ) -> Result<IndexedResults, Box<dyn std::any::Any + Send>> {
        self.handle.join()
    }
}
type IndexedResults = Vec<Indexed<Result<Data, DocumentLoadError>>>;

pub enum OptionalThread {
    NoOp,
    Thread(ProgressJoinHandle),
}

impl OptionalThread {
    pub fn create(document_paths: Vec<Indexed<PathBuf>>, parameters: &Parameters,parent_bar:MultiProgress) -> Self {
        if document_paths.is_empty() {
            return Self::NoOp;
        }
        Self::Thread(ProgressJoinHandle::new(document_paths, parameters, parent_bar))
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
    progress_bar: ProgressBar,
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
        .inspect(|_| progress_bar.inc(1))
        .collect()
}
