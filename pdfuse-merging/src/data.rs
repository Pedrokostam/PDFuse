use pdfuse_sizing::{CustomSize, Size};
use pdfuse_utils::{create_temp_dir, error_t, Indexed};
use printpdf::lopdf::{Bookmark, Document, Object, ObjectId};
use rayon::prelude::*;
use std::{
    collections::BTreeMap,
    fmt::Display,
    path::PathBuf,
};

pub use imager::Imager;
pub use loaded_document::LoadedDocument;
pub use loaded_image::LoadedImage;
use pdfuse_parameters::{
    Parameters,
    SourcePath::{self, Image, LibreDocument, Pdf},
};

use crate::DocumentLoadError;
mod imager;
mod loaded_document;
mod loaded_image;
pub enum Data {
    Image(LoadedImage),
    Document(LoadedDocument),
}

pub type PdfResult<T> = std::result::Result<T, DocumentLoadError>;
pub type IndexedPdfResult<T> = Indexed<PdfResult<T>>;

fn helpme<TSource, TTarget, TErrorSource, TErrorTarget, F>(
    value: Indexed<Result<TSource, TErrorSource>>,
    f: F,
) -> Indexed<Result<TTarget, TErrorTarget>>
where
    F: FnOnce(TSource) -> TTarget,
    TErrorSource: Into<TErrorTarget>,
{
    value.map_with_index(|indexed| indexed.map(f).map_err(Into::into))
}

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
pub fn load(sources: Vec<Indexed<SourcePath>>, parameters: &Parameters) {
    let mut images_to_load: Vec<Indexed<PathBuf>> = Vec::with_capacity(sources.len());
    let mut documents_to_pdf: Vec<Indexed<PathBuf>> = Vec::with_capacity(sources.len());
    let mut pdfs_to_load: Vec<Indexed<PathBuf>> = Vec::with_capacity(sources.len());
    for isp in sources {
        let index = isp.index();
        match isp.unwrap() {
            Image(path_buf) => images_to_load.push((index, path_buf).into()),
            Pdf(path_buf) => pdfs_to_load.push((index, path_buf).into()),
            LibreDocument(path_buf) => documents_to_pdf.push((index, path_buf).into()),
        }
    }
    let parameters_2 = parameters.clone();
    let conversion_thread = std::thread::spawn(move || {
        let paths = convert_data_to_documents(documents_to_pdf, &parameters_2);
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

    let loaded_images: Vec<IndexedPdfResult<Data>> = images_to_load
        .into_iter()
        .map(preload_image_indexed)
        .collect();
    let loaded_pdfs: Vec<IndexedPdfResult<Data>> =
        pdfs_to_load.into_iter().map(preload_pdf_indexed).collect();

    let loaded_documents = conversion_thread
        .join()
        .expect("The Libre thread should not fail");
    let mut loaded_all: Vec<IndexedPdfResult<Data>> = loaded_images
        .into_iter()
        .chain(loaded_pdfs)
        .chain(loaded_documents)
        .collect();
    loaded_all.sort_by_key(|x| x.index());
    // page size
    let mut first_page_size: Option<CustomSize> = None;
    if !parameters.force_image_page_fallback_size {
        for indexed_result in loaded_all.iter().filter(|i| i.value().is_ok()) {
            match indexed_result.value().as_ref().unwrap() {
                Data::Image(loaded_image) => continue,
                Data::Document(loaded_document) => first_page_size = loaded_document.page_size(),
            }
            if first_page_size.is_some() {
                break;
            }
        }
    }
    first_page_size.get_or_insert(parameters.image_page_fallback_size.to_custom_size());
    //
    let mut documents: Vec<Indexed<PdfResult<Document>>> = loaded_all
        .into_par_iter()
        .map(|loaded| {
            loaded.map_with_index(|indexed_result| match indexed_result {
                Ok(data) => match data {
                    Data::Image(loaded_image) => {
                        let mut imager = Imager::new(
                            "title",
                            first_page_size.unwrap(),
                            parameters.dpi,
                            parameters.margin,
                        );
                        imager.add_image(loaded_image);
                        Ok(imager.close_and_into_document())
                    }
                    Data::Document(loaded_document) => Ok(loaded_document.into()),
                },
                Err(err) => Err(err),
            })
        })
        .collect();
    documents.sort_by_key(|k| k.index());
    merge_documents(
        documents.into_iter().map(|x| x.unwrap()),
        &parameters.output_file,
    );
}

fn datify<T>(value: Indexed<T>) -> Indexed<Data>
where
    T: Into<Data>,
{
    value.map_with_index(|x| x.into())
}

fn preload_image_indexed(path: Indexed<PathBuf>) -> Indexed<PdfResult<Data>> {
    path.map_with_index(|path| LoadedImage::load(&path).map(Into::into).map_err(Into::into))
}
fn preload_pdf_indexed(path: Indexed<PathBuf>) -> Indexed<PdfResult<Data>> {
    path.map_with_index(preload_pdf)
}
fn preload_pdf(path: PathBuf) -> PdfResult<Data> {
    LoadedDocument::new_pdf(&path)
        .map(Into::into)
        .map_err(Into::into)
}
fn preload_images(image_paths: Vec<Indexed<PathBuf>>) -> Vec<Indexed<PdfResult<LoadedImage>>> {
    image_paths
        .into_iter()
        .map(|p| p.map_with_index(|path| LoadedImage::load(&path).map_err(Into::into)))
        .collect()
}

fn preload_pdfs(image_paths: Vec<Indexed<PathBuf>>) -> Vec<Indexed<PdfResult<LoadedDocument>>> {
    image_paths
        .into_iter()
        .map(|p| p.map_with_index(|path| LoadedDocument::new_pdf(&path).map_err(Into::into)))
        .collect()
}

// fn load_impl(
//     source: Indexed<SourcePath>,
//     parameters: &ParametersWithPaths,
// ) -> Option<Indexed<Data>> {
//     trace_t!("debug.loading_file", path = source);
//     let load_start = std::time::Instant::now();
//     let data: Option<Data> = match source.value() {
//         Pdf(path_buf) | LibreDocument(path_buf) => {
//             let doc_data = LoadedDocument::new(path_buf, parameters.libreoffice_path.as_deref());
//             match doc_data {
//                 Ok(document_data) => Some(Data::Document(document_data)),
//                 Err(error) => {
//                     error_t!("error.docpdf_conversion", path = path_buf.display());
//                     None
//                 }
//             }
//         }
//         Image(path_buf) => {
//             let img_data = LoadedImage::load(path_buf);
//             match img_data {
//                 Ok(loaded) => Some(Data::Image(loaded)),
//                 Err(_) => {
//                     error_t!("error.image_loading", path = path_buf.display());
//                     None
//                 }
//             }
//         }
//     };
//     let load_end = std::time::Instant::now();
//     debug_t!(
//         "debug.loaded_file_in",
//         path = source,
//         seconds = (load_end - load_start).as_secs_f32()
//     );
//     data.map(|d| Indexed::<Data>::new(source.index(), d))
// }
// pub fn merge(loaded_data: Vec<Data>, parameters: &ParametersWithPaths) {
//     let documents = convert_data_to_documents(loaded_data, parameters);
//     merge_documents(documents, &parameters.output_file);
// }
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

pub fn merge_documents<T>(documents: T, output_path: &str)
where
    T: IntoIterator<Item = PdfResult<Document>>,
{
    // Define a starting max_id (will be used as start index for object_ids)
    let mut max_id = 1;
    let mut pagenum = 1;
    // Collect all Documents Objects grouped by a map
    let mut documents_pages = BTreeMap::new();
    let mut documents_objects = BTreeMap::new();
    let mut document = Document::with_version("1.5");
    // https://github.com/J-F-Liu/lopdf/blob/0d65f6ed5b55fde1a583861535b4bfc6cdf42de1/README.md
    for result in documents {
        if result.is_err(){
            error_t!("error.image_loading",path=result.unwrap_err());
            continue;;
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
                            format!("Page_{}", pagenum),
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
        match object.type_name().unwrap_or("") {
            "Catalog" => {
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
            "Pages" => {
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
            "Page" => {}     // Ignored, processed later and separately
            "Outlines" => {} // Ignored, not supported yet
            "Outline" => {}  // Ignored, not supported yet
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

pub fn do_everything(files: Vec<Indexed<SourcePath>>, parameters: &Parameters) {
    load(files, parameters);
}
