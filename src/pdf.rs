use std::collections::BTreeMap;

use log::error;
use lopdf::{Bookmark, Document, Object, ObjectId};
pub mod image_pdf;
use image_pdf::ImagePdf;

use crate::error_t;

pub enum Pdf {
     Images(ImagePdf),
    PdfFile(Document),
    //OdfFile
}

impl Pdf {
    pub fn into_document(self) -> Document {
        match self {
            Pdf::Images(image_pdf) => image_pdf.into_document(),
            Pdf::PdfFile(document) => document,
        }
    }
}
impl From<Document> for Pdf{
    fn from(value: Document) -> Self {
        Pdf::PdfFile(value)
    }
}
impl From<ImagePdf> for Pdf{
    fn from(value: ImagePdf) -> Self {
        Pdf::Images(value)
    }
}
unsafe impl Send for Pdf{}
unsafe impl Sync for Pdf{}
pub fn merge_pdfs(pdfs: Vec<Pdf>, output_path: &str) {
    // Define a starting max_id (will be used as start index for object_ids)
    let mut max_id = 1;
    let mut pagenum = 1;
    // Collect all Documents Objects grouped by a map
    let mut documents_pages = BTreeMap::new();
    let mut documents_objects = BTreeMap::new();
    let mut document = Document::with_version("1.5");
    // https://github.com/J-F-Liu/lopdf/blob/0d65f6ed5b55fde1a583861535b4bfc6cdf42de1/README.md
    for pdf in pdfs {
        let mut first = false;
        let mut doc = pdf.into_document();

        doc.renumber_objects_with(max_id);
        max_id = doc.max_id + 1;

        documents_pages.extend(
            doc.get_pages().into_values().map(|object_id| {
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
                    (
                        object_id,
                        doc.get_object(object_id).unwrap().to_owned(),
                    )
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
        error_t!("debug.root_not_found",item="Page");
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
        error_t!("debug.root_not_found",item="Catalog");
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
            documents_pages.into_keys().map(Object::Reference)
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
    if let Some(n) = document.build_outline()  {
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
