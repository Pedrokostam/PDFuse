use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use lopdf::{Bookmark, Document, Object, ObjectId};
use pdfuse_sizing::{CustomSize, PageSize, Size};
use pdfuse_utils::{create_temp_dir, error_t, get_progress_indicator, log, BusyIndicator, Indexed};
use rayon::prelude::*;
use size_guide::SizeGuide;
use std::{collections::BTreeMap, fmt::Display, path::PathBuf, time::Duration};

pub use imager::Imager;
pub use loaded_document::LoadedDocument;
pub use loaded_image::LoadedImage;
use pdfuse_parameters::{
    source_path::display_path, Parameters, SourcePath::{self, Image, LibreDocument, Pdf}
};

use crate::DocumentLoadError;
mod imager;
mod loaded_document;
mod loaded_image;
mod optional_thread;
mod size_guide;
use optional_thread::OptionalThread;

/// Applies `f` to each element of `iter` and collects the results into a `Vec`
fn vector_map<T, U, F, W>(iter: T, f: F) -> Vec<W>
where
    T: IntoIterator<Item = U>,
    F: FnMut(U) -> W,
{
    iter.into_iter().map(f).collect()
}

pub enum Data {
    Image(LoadedImage),
    Document(LoadedDocument),
}

pub type PdfResult<T> = std::result::Result<T, DocumentLoadError>;
pub type IndexedPdfResult<T> = Indexed<PdfResult<T>>;

impl From<LoadedImage> for Data {
    fn from(value: LoadedImage) -> Self {
        Self::Image(value)
    }
}

impl From<LoadedDocument> for Data {
    fn from(value: LoadedDocument) -> Self {
        Self::Document(value)
    }
}

impl Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::Image(image_data) => write!(
                f,
                "Image data: {w}x{h}, from \"{path}\"",
                w = image_data.width(),
                h = image_data.height(),
                path = image_data.source_path().display()
            ),
            Data::Document(document_data) => document_data.fmt(f),
        }
    }
}

struct SplitPathsResult(
    Vec<Indexed<PathBuf>>,
    Vec<Indexed<PathBuf>>,
    Vec<Indexed<PathBuf>>,
);

fn split_paths(sources: Vec<Indexed<SourcePath>>) -> SplitPathsResult {
    let mut images_to_load: Vec<Indexed<PathBuf>> = Vec::with_capacity(sources.len());
    let mut pdfs_to_load: Vec<Indexed<PathBuf>> = Vec::with_capacity(sources.len());
    let mut documents_to_pdf: Vec<Indexed<PathBuf>> = Vec::with_capacity(sources.len());
    for isp in sources {
        let index = isp.index();
        match isp.unwrap() {
            Image(path_buf) => images_to_load.push((index, path_buf).into()),
            Pdf(path_buf) => pdfs_to_load.push((index, path_buf).into()),
            LibreDocument(path_buf) => documents_to_pdf.push((index, path_buf).into()),
        }
    }
    SplitPathsResult(images_to_load, pdfs_to_load, documents_to_pdf)
}
fn size_information_not_needed(
    loaded_images: Vec<IndexedPdfResult<Data>>,
    loaded_pdfs: Vec<IndexedPdfResult<Data>>,
    parameters: &Parameters,
    optional_thread: OptionalThread,
    multi_progress: &MultiProgress,
) -> Vec<IndexedPdfResult<Document>> {
    run_in_parallel_with_libre(
        loaded_images,
        loaded_pdfs,
        parameters,
        optional_thread,
        multi_progress,
    )
}
/// Images (if any) do not need to rely on libre documents' sizes
/// It's possible to start conversion while the libre thread is running
fn run_in_parallel_with_libre(
    loaded_images: Vec<IndexedPdfResult<Data>>,
    loaded_pdfs: Vec<IndexedPdfResult<Data>>,
    parameters: &Parameters,
    optional_thread: OptionalThread,
    multi_progress: &MultiProgress,
) -> Vec<IndexedPdfResult<Document>> {
    let loaded_images_pdfs: Vec<IndexedPdfResult<Data>> =
        loaded_images.into_iter().chain(loaded_pdfs).collect();

    let guide = SizeGuide::new(&loaded_images_pdfs, parameters);

    let images_and_pdfs: Vec<Indexed<Result<Document, DocumentLoadError>>> =
        parallel_documentize(parameters, &guide, loaded_images_pdfs, &multi_progress);

    // join the parallel thread now, after converting all images
    let converted_documents = optional_thread.get_converted_data();
    let loaded_documents =
        parallel_documentize(parameters, &guide, converted_documents, &multi_progress);

    let mut all_items: Vec<IndexedPdfResult<Document>> = images_and_pdfs
        .into_iter()
        .chain(loaded_documents)
        .collect();
    all_items.sort_by_key(|x| x.index());
    all_items
}

/// Images need to rely on libre documents' sizes
/// Have to wait until libre conversion is done
fn wait_for_libre(
    loaded_images: Vec<IndexedPdfResult<Data>>,
    loaded_pdfs: Vec<IndexedPdfResult<Data>>,
    parameters: &Parameters,
    optional_thread: OptionalThread,
    multi_progress: &MultiProgress,
) -> Vec<IndexedPdfResult<Document>> {
    let loaded_converted_documents: Vec<IndexedPdfResult<Data>> =
        optional_thread.get_converted_data();

    let loaded_all: Vec<IndexedPdfResult<Data>> = loaded_pdfs
        .into_iter()
        .chain(loaded_images)
        .chain(loaded_converted_documents)
        .collect();

    let guide = SizeGuide::new(&loaded_all, parameters);

    let mut all_items = parallel_documentize(parameters, &guide, loaded_all, multi_progress);
    all_items.sort_by_key(|x| x.index());
    all_items
}

fn parallel_documentize(
    parameters: &Parameters,
    guide: &SizeGuide,
    loaded_all: Vec<Indexed<Result<Data, DocumentLoadError>>>,
    multi_progress: &MultiProgress,
) -> Vec<Indexed<Result<Document, DocumentLoadError>>> {
    let bar = multi_progress.add(get_progress_indicator(
        loaded_all.len() as u64,
        "Converting images and PDFs",
    ));
    loaded_all
        .into_par_iter()
        .map(|loaded| {
            let index = loaded.index();
            let value = match loaded.unwrap() {
                Ok(data) => match data {
                    Data::Image(loaded_image) => {
                        let mut imager = Imager::new(
                            "title",
                            guide.get_size(index),
                            parameters.image_dpi,
                            parameters.margin,
                            parameters.image_quality,
                            parameters.image_lossless_compression,
                        );
                        let path = display_path(loaded_image.source_path());
                        match imager.add_image(loaded_image){
                            Ok(_) => (),
                            Err(e) => log::error!("{e} - {path}"),
                        }
                        Ok(imager.close_and_into_document())
                    }
                    Data::Document(loaded_document) => Ok(loaded_document.into()),
                },
                Err(err) => Err(err),
            };
            Indexed::new(index, value)
        })
        .inspect(|_| bar.inc(1))
        .collect()
}

pub fn load(sources: Vec<Indexed<SourcePath>>, parameters: &Parameters) {
    if !sources.is_sorted_by_key(|x| x.index()) {
        panic!("Paths are supposed to be sorted already!");
    }
    let parent_bar = MultiProgress::new();
    // let busy = BusyIndicator::new_with_message("Loading files...");
    let branch = SizeGuide::need_to_wait_for_pdf_threads(&sources, parameters);
    let SplitPathsResult(images_to_load, pdfs_to_load, documents_to_pdf) = split_paths(sources);

    let conversion_thread =
        OptionalThread::create(documents_to_pdf, parameters, parent_bar.clone());
    // load all PDFs as Data - limited only by disk IO
    let loaded_pdfs = vector_map(pdfs_to_load, preload_pdf_indexed);

    // load all images as Data - limited only by disk IO
    let loaded_images: Vec<IndexedPdfResult<Data>> =
        vector_map(images_to_load, preload_image_indexed);

    // drop(busy);
    let all_documents_to_merge = match branch {
        size_guide::GuideRequirement::SizeInformationNotNeeded => size_information_not_needed(
            loaded_images,
            loaded_pdfs,
            parameters,
            conversion_thread,
            &parent_bar,
        ),
        size_guide::GuideRequirement::WaitForLibreConversion => wait_for_libre(
            loaded_images,
            loaded_pdfs,
            parameters,
            conversion_thread,
            &parent_bar,
        ),
        size_guide::GuideRequirement::RunInParallelWithLibreConversion => {
            run_in_parallel_with_libre(
                loaded_images,
                loaded_pdfs,
                parameters,
                conversion_thread,
                &parent_bar,
            )
        }
    };
    merge_documents(
        all_documents_to_merge.into_iter().map(|x| x.unwrap()),
        &parameters.output_file,
    );
}

fn preload_image_indexed(path: Indexed<PathBuf>) -> Indexed<PdfResult<Data>> {
    path.map_with_index(|path| LoadedImage::load(&path).map(Into::into).map_err(Into::into))
}
fn preload_pdf_indexed(path: Indexed<PathBuf>) -> Indexed<PdfResult<Data>> {
    path.map_with_index(preload_pdf)
}
fn preload_pdf(path: PathBuf) -> PdfResult<Data> {
    LoadedDocument::load_pdf(&path).map(LoadedDocument::into)
}

pub fn merge_documents<T>(documents: T, output_path: &str)
where
    T: IntoIterator<Item = PdfResult<Document>>,
{
    // Define a starting max_id (will be used as start index for object_ids)
    let mut max_id = 1;
    let mut pagenum = 1;
    // Collect all Documents Objects grouped by a map
    let mut documents_pages: BTreeMap<ObjectId, Object> = BTreeMap::new();
    let mut documents_objects: BTreeMap<ObjectId, Object> = BTreeMap::new();
    let mut document = Document::with_version("1.5");
    // https://github.com/J-F-Liu/lopdf/blob/0d65f6ed5b55fde1a583861535b4bfc6cdf42de1/README.md
    for result in documents {
        if result.is_err() {
            error_t!("error.image_loading", path = result.unwrap_err());
            continue;
        }
        let mut doc = result.unwrap();
        let mut first = false;

        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;

        documents_pages.extend(
            doc.get_pages()
                .into_values()
                .map(|object_id| {
                    if !first {
                        let bookmark = Bookmark::new(
                            format!("Page_{pagenum}"),
                            [0.0, 0.0, 1.0],
                            0,
                            object_id,
                        );
                        document.add_bookmark(bookmark, None);
                        first = true;
                        pagenum += 1;
                    }
                    (object_id, doc.get_object(object_id).unwrap().to_owned())
                })
                .collect::<BTreeMap<ObjectId, Object>>(),
        );
        documents_objects.extend(doc.objects);
    }

    // Catalog and Pages are mandatory
    let mut catalog_object: Option<(ObjectId, Object)> = None;
    let mut pages_object: Option<(ObjectId, Object)> = None;

    // Process all objects except "Page" type
    for (object_id, object) in documents_objects.iter() {
        // We have to ignore "Page" (as are processed later), "Outlines" and "Outline" objects
        // All other objects should be collected and inserted into the main Document
        match object.type_name().unwrap_or(b"") {
            b"Catalog" => {
                // Collect a first "Catalog" object and use it for the future "Pages"
                catalog_object = Some((
                    if let Some((id, _)) = catalog_object {
                        id
                    } else {
                        *object_id
                    },
                    object.clone(),
                ));
            }
            b"Pages" => {
                // Collect and update a first "Pages" object and use it for the future "Catalog"
                // We have also to merge all dictionaries of the old and the new "Pages" object
                if let Ok(dictionary) = object.as_dict() {
                    let mut dictionary = dictionary.clone();
                    if let Some((_, ref object)) = pages_object {
                        if let Ok(old_dictionary) = object.as_dict() {
                            dictionary.extend(old_dictionary);
                        }
                    }

                    pages_object = Some((
                        if let Some((id, _)) = pages_object {
                            id
                        } else {
                            *object_id
                        },
                        Object::Dictionary(dictionary),
                    ));
                }
            }
            b"Page" => {}     // Ignored, processed later and separately
            b"Outlines" => {} // Ignored, not supported yet
            b"Outline" => {}  // Ignored, not supported yet
            _ => {
                document.objects.insert(*object_id, object.clone());
            }
        }
    }

    // If no "Pages" object found abort
    if pages_object.is_none() {
        error_t!("debug.root_not_found", item = "Page");
        return;
    }
    // Iterate over all "Page" objects and collect into the parent "Pages" created before
    for (object_id, object) in documents_pages.iter() {
        if let Ok(dictionary) = object.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set("Parent", pages_object.as_ref().unwrap().0);

            document
                .objects
                .insert(*object_id, Object::Dictionary(dictionary));
        }
    }

    // If no "Catalog" found abort
    if catalog_object.is_none() {
        error_t!("debug.root_not_found", item = "Catalog");
        return;
    }

    let catalog_object = catalog_object.unwrap();
    let pages_object = pages_object.unwrap();

    // Build a new "Pages" with updated fields
    if let Ok(dictionary) = pages_object.1.as_dict() {
        let mut dictionary = dictionary.clone();

        // Set new pages count
        dictionary.set("Count", documents_pages.len() as u32);

        // Set new "Kids" list (collected from documents pages) for "Pages"
        dictionary.set(
            "Kids",
            documents_pages
                .into_keys()
                .map(Object::Reference)
                .collect::<Vec<_>>(),
        );

        document
            .objects
            .insert(pages_object.0, Object::Dictionary(dictionary));
    }

    // Build a new "Catalog" with updated fields
    if let Ok(dictionary) = catalog_object.1.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Pages", pages_object.0);
        dictionary.remove(b"Outlines"); // Outlines not supported in merged PDFs

        document
            .objects
            .insert(catalog_object.0, Object::Dictionary(dictionary));
    }

    document.trailer.set("Root", catalog_object.0);

    // Update the max internal ID as wasn't updated before due to direct objects insertion
    document.max_id = document.objects.len() as u32;

    // Reorder all new Document objects
    document.renumber_objects();

    //Set any Bookmarks to the First child if they are not set to a page
    document.adjust_zero_pages();

    //Set all bookmarks to the PDF Object tree then set the Outlines to the Bookmark content map.
    if let Some(n) = document.build_outline() {
        if let Ok(Object::Dictionary(ref mut dict)) = document.get_object_mut(catalog_object.0) {
            dict.set("Outlines", Object::Reference(n));
        }
    }

    document.compress();

    document.save(output_path).unwrap();
    // Save the merged PDF
    // Store file in current working directory.
    // Note: Line is excluded when running tests
}
