use indicatif::{ProgressBar, ProgressIterator};
use lopdf::{Bookmark, Document, Object, ObjectId};
use pdfuse_utils::{
    error_t, get_registered_busy_indicator, get_registered_progress_iterator,
    get_registered_progress_iterator_parallel,
    log::{debug, error},
    register_progressbar, Indexed,
};
use rayon::iter::ParallelIterator;
use size_guide::SizeGuide;
use std::{collections::BTreeMap, env, error::Error, fmt::Display, path::Path};

pub use imager::Imager;
pub use loaded_document::LoadedDocument;
pub use loaded_image::LoadedImage;
use pdfuse_parameters::{
    Bookmarks, Parameters, SafePath,
    SourcePath::{self, Image, LibreDocument, Pdf},
};

use crate::{conditional_slow_down, DocumentLoadError};
mod imager;
mod loaded_document;
mod loaded_image;
mod optional_thread;
mod size_guide;
use optional_thread::OptionalThread;

/// Applies `f` to each element of `iter` and collects the results into a `Vec`
fn vector_map<U, F, W>(iter: Vec<U>, f: F, message: &str) -> Vec<W>
where
    F: FnMut(U) -> W,
{
    if iter.is_empty() {
        return Default::default();
    }
    get_registered_progress_iterator(iter.into_iter(), message.to_owned())
        .map(f)
        .collect()
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

/// Named struct containing 3 vector of SourcePaths, divided by their type.
struct SplitPathsResult {
    images: Vec<Indexed<SafePath>>,
    pdfs: Vec<Indexed<SafePath>>,
    docs: Vec<Indexed<SafePath>>,
}
/// Splits given iterable of indexed `SourcePath`s into 3 vectors and puts them into a named struct.
///
/// # Panics
///
/// Panics if .
///
/// # Errors
///
/// This function will return an error if .
fn split_paths(sources: Vec<Indexed<SourcePath>>) -> SplitPathsResult {
    let mut images_to_load: Vec<Indexed<SafePath>> = Vec::with_capacity(sources.len());
    let mut pdfs_to_load: Vec<Indexed<SafePath>> = Vec::with_capacity(sources.len());
    let mut documents_to_pdf: Vec<Indexed<SafePath>> = Vec::with_capacity(sources.len());
    for isp in sources {
        let index = isp.index();
        match isp.take_out() {
            Image(spath) => images_to_load.push((index, spath).into()),
            Pdf(spath) => pdfs_to_load.push((index, spath).into()),
            LibreDocument(spath) => documents_to_pdf.push((index, spath).into()),
        }
    }
    SplitPathsResult {
        images: images_to_load,
        pdfs: pdfs_to_load,
        docs: documents_to_pdf,
    }
}

/// Like <code>run_in_parallel_with_libre</code> but when there are no documents or if the fallback size is forced.
fn size_information_not_needed(
    loaded_images: Vec<IndexedPdfResult<Data>>,
    loaded_pdfs: Vec<IndexedPdfResult<Data>>,
    parameters: &Parameters,
    optional_thread: OptionalThread,
) -> Vec<IndexedPdfResult<LoadedDocument>> {
    debug!("size_information_not_needed");
    run_in_parallel_with_libre(loaded_images, loaded_pdfs, parameters, optional_thread)
}
/// Images (if any) do not need to rely on libre documents' sizes.
///
/// It's possible to start image conversion while the libre thread is running
fn run_in_parallel_with_libre(
    loaded_images: Vec<IndexedPdfResult<Data>>,
    loaded_pdfs: Vec<IndexedPdfResult<Data>>,
    parameters: &Parameters,
    optional_thread: OptionalThread,
) -> Vec<IndexedPdfResult<LoadedDocument>> {
    debug!("run_in_parallel_with_libre");
    let mut loaded_images_pdfs: Vec<IndexedPdfResult<Data>> =
        loaded_images.into_iter().chain(loaded_pdfs).collect();

    let guide = SizeGuide::new(&mut loaded_images_pdfs, parameters);

    let images_and_pdfs = documentize(parameters, &guide, loaded_images_pdfs, "_&IMG PDF");

    // join the parallel thread now, after converting all images
    let converted_documents = optional_thread.get_converted_data();
    let loaded_documents = documentize(parameters, &guide, converted_documents, "_&IMG PDF DOC");

    images_and_pdfs
        .into_iter()
        .chain(loaded_documents)
        .collect()
}

/// Images need to rely on libre documents' sizes.
///
/// Image conversion has to wait until libre conversion finishes.
fn wait_for_libre(
    loaded_images: Vec<IndexedPdfResult<Data>>,
    loaded_pdfs: Vec<IndexedPdfResult<Data>>,
    parameters: &Parameters,
    optional_thread: OptionalThread,
) -> Vec<IndexedPdfResult<LoadedDocument>> {
    debug!("wait_for_libre",);
    let loaded_converted_documents: Vec<IndexedPdfResult<Data>> =
        optional_thread.get_converted_data();

    let mut loaded_all: Vec<IndexedPdfResult<Data>> = loaded_pdfs
        .into_iter()
        .chain(loaded_images)
        .chain(loaded_converted_documents)
        .collect();

    let guide = SizeGuide::new(&mut loaded_all, parameters);

    documentize(parameters, &guide, loaded_all, "_& IMG PDF DOC")
}

fn sequential_documentize(
    parameters: &Parameters,
    guide: &SizeGuide,
    loaded_all: Vec<IndexedPdfResult<Data>>,
    message: &str,
) -> Vec<Indexed<Result<LoadedDocument, DocumentLoadError>>> {
    debug!("parallel_documentize");
    let iterator =
        get_registered_progress_iterator(loaded_all.into_iter(), message.to_owned() + " _SEQ");
    let mut imager: Option<Imager> = None;
    let mut output: Vec<Indexed<Result<LoadedDocument, DocumentLoadError>>> = vec![];
    let mut index = 0;
    for a in iterator {
        index = a.index();
        let _ = match a.take_out() {
            Err(e) => Err(e),
            Ok(item) => {
                match item {
                    Data::Document(loaded_document) => {
                        if let Some(img) = imager {
                            let new_images = LoadedDocument::from_document_like(
                                Default::default(),
                                Box::new(img.close_and_into_document()),
                            );
                            output.push(Indexed::new(index - 1, Ok(new_images)));
                            imager = None;
                        };
                        output.push(Indexed::new(index, Ok(loaded_document)));
                    }
                    Data::Image(loaded_image) => {
                        let refimg = imager.get_or_insert_with(|| {
                            Imager::new(
                                "title",
                                guide.get_size(index),
                                parameters.image_dpi,
                                parameters.margin,
                                parameters.image_quality,
                                parameters.image_lossless_compression,
                            )
                        });
                        match refimg.add_image(loaded_image) {
                            Ok(_) => (),
                            Err(e) => {
                                output.push(Indexed::new(index, Err(e.into())));
                                imager = None;
                            }
                        };
                    }
                };
                Ok(())
            }
        };
    }
    if let Some(img) = imager {
        let new_images = LoadedDocument::from_document_like(
            Default::default(),
            Box::new(img.close_and_into_document()),
        );
        output.push(Indexed::new(index, Ok(new_images)));
    }
    output
}

fn parallel_documentize(
    parameters: &Parameters,
    guide: &SizeGuide,
    loaded_all: Vec<IndexedPdfResult<Data>>,
    message: &str,
) -> Vec<Indexed<Result<LoadedDocument, DocumentLoadError>>> {
    let iterator =
        get_registered_progress_iterator_parallel(loaded_all, message.to_owned() + " PAR");
    iterator
        .map(|loaded| {
            conditional_slow_down();
            let index = loaded.index();
            let value = match loaded.take_out() {
                Err(e) => Err(e),
                Ok(item) => match item {
                    Data::Document(loaded_document) => Ok(loaded_document),
                    Data::Image(loaded_image) => {
                        one_image_imager(loaded_image, parameters, guide, index)
                    }
                },
            };
            Indexed::new(index, value)
        })
        .collect()
}

fn parallel_documentize_seq(
    parameters: &Parameters,
    guide: &SizeGuide,
    loaded_all: Vec<IndexedPdfResult<Data>>,
    message: &str,
) -> Vec<Indexed<Result<LoadedDocument, DocumentLoadError>>> {
    let iterator =
        get_registered_progress_iterator(loaded_all.into_iter(), message.to_owned() + " PAR");
    iterator
        .map(|loaded| {
            conditional_slow_down();
            let index = loaded.index();
            let value = match loaded.take_out() {
                Err(e) => Err(e),
                Ok(item) => match item {
                    Data::Document(loaded_document) => Ok(loaded_document),
                    Data::Image(loaded_image) => {
                        one_image_imager(loaded_image, parameters, guide, index)
                    }
                },
            };
            Indexed::new(index, value)
        })
        .collect()
}

fn documentize(
    parameters: &Parameters,
    guide: &SizeGuide,
    loaded_all: Vec<IndexedPdfResult<Data>>,
    message: &str,
) -> Vec<Indexed<Result<LoadedDocument, DocumentLoadError>>> {
    if let Ok(val) = env::var("PARALLEL") {
        if val == "SEQ" {
            debug!("PARALLEL_SEQ");
            parallel_documentize_seq(parameters, guide, loaded_all, message)
        } else {
            debug!("PARALLEL");
            parallel_documentize(parameters, guide, loaded_all, message)
        }
    } else {
        debug!("SEQUENTIAL");
        sequential_documentize(parameters, guide, loaded_all, message)
    }
}

fn one_image_imager(
    image: LoadedImage,
    parameters: &Parameters,
    guide: &SizeGuide,
    index: usize,
) -> Result<LoadedDocument, DocumentLoadError> {
    let mut refimg = Imager::new(
        "title",
        guide.get_size(index),
        parameters.image_dpi,
        parameters.margin,
        parameters.image_quality,
        parameters.image_lossless_compression,
    );
    let source_path = image.source_path().to_owned();
    match refimg.add_image(image) {
        Ok(_) => Ok(LoadedDocument::from_document_like(
            source_path,
            Box::new(refimg.close_and_into_document()),
        )),
        Err(e) => Err(e.into()),
    }
}
/*
Ok(data) => match data {
               Data::Image(loaded_image) => {
               let refimg = imager.get_or_insert_with(||Imager::new(
                       "title",
                       guide.get_size(index),
                       parameters.image_dpi,
                       parameters.margin,
                       parameters.image_quality,
                       parameters.image_lossless_compression,
                   ));
                   let path = loaded_image.source_path().to_display_string();
                   match refimg.add_image(loaded_image) {
                       Ok(_) => (),
                       Err(e) => log::error!("{e} - {path}"),
                   }
                   Ok(LoadedDocument(imager.close_and_into_document()))
               }
               Data::Document(loaded_document) => Ok(loaded_document),
           },
           Err(err) => Err(err),
*/
pub fn load(sources: Vec<Indexed<SourcePath>>, parameters: &Parameters) {
    if !sources.is_sorted_by_key(|x| x.index()) {
        panic!("Paths are supposed to be sorted already!");
    }
    // let busy = BusyIndicator::new_with_message("Loading files...");
    let branch = SizeGuide::need_to_wait_for_pdf_threads(&sources, parameters);
    let SplitPathsResult {
        images: images_to_load,
        pdfs: pdfs_to_load,
        docs: documents_to_pdf,
    } = split_paths(sources);

    let conversion_thread = OptionalThread::create(documents_to_pdf, parameters);
    // load all PDFs as Data - limited only by disk IO
    let loaded_pdfs = vector_map(pdfs_to_load, preload_pdf_indexed, "_&Preloading PDFs");

    // load all images as Data - limited only by disk IO
    let loaded_images: Vec<IndexedPdfResult<Data>> =
        vector_map(images_to_load, preload_image_indexed, "_&Preloading images");

    // drop(busy);
    let mut all_documents_to_merge = match branch {
        size_guide::GuideRequirement::SizeInformationNotNeeded => {
            size_information_not_needed(loaded_images, loaded_pdfs, parameters, conversion_thread)
        }
        size_guide::GuideRequirement::WaitForLibreConversion => {
            wait_for_libre(loaded_images, loaded_pdfs, parameters, conversion_thread)
        }
        size_guide::GuideRequirement::RunInParallelWithLibreConversion => {
            run_in_parallel_with_libre(loaded_images, loaded_pdfs, parameters, conversion_thread)
        }
    };
    all_documents_to_merge.sort_unstable();
    merge_documents(
        all_documents_to_merge.into_iter(),
        &parameters.output_file,
        parameters.bookmarks,
    );
}

fn inspect_err(error: &impl Error) {
    error!("{error}")
}

fn preload_image_indexed(path: Indexed<SafePath>) -> Indexed<PdfResult<Data>> {
    path.map_with_index(|path| {
        LoadedImage::load(&path)
            .map(Into::into)
            .map_err(Into::into)
            .inspect_err(inspect_err)
    })
}
fn preload_pdf_indexed(path: Indexed<SafePath>) -> Indexed<PdfResult<Data>> {
    path.map_with_index(preload_pdf)
}
fn preload_pdf(path: SafePath) -> PdfResult<Data> {
    conditional_slow_down();
    LoadedDocument::load_pdf(&path)
        .map(LoadedDocument::into)
        .inspect_err(inspect_err)
}

// fn add_page_objects(
//     loaded_docs: impl Iterator<Item = IndexedPdfResult<LoadedDocument>>,
//     bookmark: Bookmarks,
//     output_document_pages: &mut BTreeMap<ObjectId, Object>,
//     output_document_objects: &mut BTreeMap<ObjectId, Object>,
//     output_document: &mut Document,
//     max_id: &mut u32,
//     first_page_of_doc:&mut bool,
// ) {
//     let mut errors: Vec<usize> = vec![];
//     for result in loaded_docs {
//         if result.value().is_err() {
//             errors.push(result.index());
//             return;
//         }
//         let (index, res_loaded_document) = result.deconstruct();
//         let loaded_document = res_loaded_document.expect("Already checked for errors");
//         let path = loaded_document.source_path().to_owned();
//         let mut iterated_document: Document = loaded_document.into();
//         iterated_document.renumber_objects_with(*max_id);
//         *max_id = iterated_document.max_id + 1;
//         let mut output_bookmark_parent: Option<u32> = None;
//         output_document_pages.extend(iterated_document.get_pages().into_values().map(
//             |object_id| {
//                 if *first_page_of_doc {
//                     *first_page_of_doc = false;
//                     let bookmark_text = match bookmark {
//                         Bookmarks::None => None,
//                         Bookmarks::Index => Some(format!("{index}")),
//                         Bookmarks::IndexName => Some(format!("{} - {}", index, path.file_name())),
//                     };
//                     if let Some(txt) = bookmark_text {
//                         let bookmark = Bookmark::new(txt, [0.0, 0.0, 1.0], 0, object_id);
//                         let new_parent = output_document.add_bookmark(bookmark, None);
//                         output_bookmark_parent = Some(new_parent);
//                     }
//                 }
//                 (
//                     object_id,
//                     iterated_document.get_object(object_id).unwrap().to_owned(),
//                 )
//             },
//         ));
//         for (_, existing_bookmark) in iterated_document.bookmark_table {
//             output_document.add_bookmark(existing_bookmark, output_bookmark_parent);
//         }
//         output_bookmark_parent = None;
//         first_page_of_doc = true;
//         output_document_objects.extend(iterated_document.objects);
//     }
// }

// pub fn merge_documents<T>(documents: T, output_path: &Path)
pub fn merge_documents<T>(documents: T, output_path: &Path, bookmark: Bookmarks)
where
    T: IntoIterator<Item = IndexedPdfResult<LoadedDocument>> + ExactSizeIterator,
{
    // Define a starting max_id (will be used as start index for object_ids)
    let busy = get_registered_busy_indicator("_&Generating PDF...");
    let mut max_id = 1;
    // Collect all Documents Objects grouped by a map
    let mut output_documents_pages: BTreeMap<ObjectId, Object> = BTreeMap::new();
    let mut output_documents_objects: BTreeMap<ObjectId, Object> = BTreeMap::new();
    let mut output_document = Document::with_version("1.5");
    // https://github.com/J-F-Liu/lopdf/blob/0d65f6ed5b55fde1a583861535b4bfc6cdf42de1/README.md
    let mut errors: Vec<usize> = vec![];
    let iterator = documents.into_iter();
    let mut first_page_of_doc = true;
    for result in iterator {
        if result.value().is_err() {
            errors.push(result.index());
            continue;
        }
        let (index, res_loaded_document) = result.deconstruct();
        let loaded_document = res_loaded_document.expect("Already checked for errors");
        let path = loaded_document.source_path().to_owned();
        let mut iterated_document: Document = loaded_document.into();
        iterated_document.renumber_objects_with(max_id);
        max_id = iterated_document.max_id + 1;
        let mut output_bookmark_parent: Option<u32> = None;
        output_documents_pages.extend(iterated_document.get_pages().into_values().map(
            |object_id| {
                if first_page_of_doc {
                    first_page_of_doc = false;
                    let bookmark_text = match bookmark {
                        Bookmarks::None => Some("A".to_owned()),
                        Bookmarks::Index => Some(format!("{index}")),
                        Bookmarks::IndexName => Some(format!("{} - {}", index, path.file_name())),
                    };
                    if let Some(txt) = bookmark_text {
                        let bookmark = Bookmark::new(txt, [0.0, 0.0, 1.0], 0, object_id);
                        let new_parent = output_document.add_bookmark(bookmark, None);
                        output_bookmark_parent = Some(new_parent);
                    }
                }
                (
                    object_id,
                    iterated_document.get_object(object_id).unwrap().to_owned(),
                )
            },
        ));
        debug!("{output_bookmark_parent:?}");
        for (_, existing_bookmark) in iterated_document.bookmark_table {
            debug!("{existing_bookmark:?}");
            output_document.add_bookmark(existing_bookmark, output_bookmark_parent);
        }
        output_bookmark_parent = None;
        first_page_of_doc = true;
        output_documents_objects.extend(iterated_document.objects);
    }

    // Catalog and Pages are mandatory
    let mut catalog_object: Option<(ObjectId, Object)> = None;
    let mut pages_object: Option<(ObjectId, Object)> = None;

    // Process all objects except "Page" type
    for (object_id, object) in output_documents_objects.iter() {
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
                output_document.objects.insert(*object_id, object.clone());
            }
        }
    }

    // If no "Pages" object found abort
    if pages_object.is_none() {
        error_t!("debug.root_not_found", item = "Page");
        return;
    }
    // Iterate over all "Page" objects and collect into the parent "Pages" created before
    for (object_id, object) in output_documents_pages.iter() {
        if let Ok(dictionary) = object.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set("Parent", pages_object.as_ref().unwrap().0);

            output_document
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
        dictionary.set("Count", output_documents_pages.len() as u32);

        // Set new "Kids" list (collected from documents pages) for "Pages"
        dictionary.set(
            "Kids",
            output_documents_pages
                .into_keys()
                .map(Object::Reference)
                .collect::<Vec<_>>(),
        );

        output_document
            .objects
            .insert(pages_object.0, Object::Dictionary(dictionary));
    }

    // Build a new "Catalog" with updated fields
    if let Ok(dictionary) = catalog_object.1.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Pages", pages_object.0);
        dictionary.remove(b"Outlines"); // Outlines not supported in merged PDFs

        output_document
            .objects
            .insert(catalog_object.0, Object::Dictionary(dictionary));
    }

    output_document.trailer.set("Root", catalog_object.0);

    // Update the max internal ID as wasn't updated before due to direct objects insertion
    output_document.max_id = output_document.objects.len() as u32;

    // Reorder all new Document objects
    output_document.renumber_objects();

    //Set any Bookmarks to the First child if they are not set to a page
    output_document.adjust_zero_pages();

    //Set all bookmarks to the PDF Object tree then set the Outlines to the Bookmark content map.
    if let Some(n) = output_document.build_outline() {
        if let Ok(Object::Dictionary(ref mut dict)) =
            output_document.get_object_mut(catalog_object.0)
        {
            dict.set("Outlines", Object::Reference(n));
        }
    }

    output_document.compress();
    output_document.save(output_path).unwrap();
    if !errors.is_empty() {
        let indices = errors
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        error_t!(
            "error.failed_count",
            count = errors.len(),
            indices = indices
        );
    }
    busy.finish_and_clear();
    // Save the merged PDF
    // Store file in current working directory.
    // Note: Line is excluded when running tests
}
