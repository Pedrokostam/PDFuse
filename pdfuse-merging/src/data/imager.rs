use image::{imageops::FilterType, DynamicImage};
use lopdf::{dictionary, Document};
use pdfuse_parameters::path::SourcePath;
use pdfuse_sizing::paper::CustomPage;
use pdfuse_sizing::{Length, Size};
use pdfuse_utils::debug_t;
use pdfuse_utils::log::{debug, error};

use crate::conditional_slow_down;
use crate::data::document_source::DocumentSources;
use crate::data::loaded_image::LoadedImage2;
use crate::data::LoadedDocument;
use crate::error::ImageLoadError;

use super::LoadedImage;

/// An image with its computed on-page size and lower-left position, both in
/// PDF points (bottom-left origin).
struct PlacedImage {
    image: LoadedImage2,
    size: CustomPage,
    position: CustomPage,
}

pub struct Imager {
    pub(crate) page_size: CustomPage,
    pub(crate) dpi: f64,
    pub(crate) margin: CustomPage,
    pub(crate) quality: u8,
    pub(crate) lossless: bool,
    page_paths: Vec<SourcePath>,
    placements: Vec<PlacedImage>,
}
impl Imager {
    pub fn close_and_into_loaded_document(self) -> LoadedDocument {
        let doc_sources = DocumentSources::new_multi(self.page_paths.clone());
        let doc = self.close_and_into_document();
        LoadedDocument::from_document_like(doc_sources, Box::new(doc))
    }

    fn close_and_into_document(self) -> Document {
        let mut doc = Document::new();
        let pages_root = doc.new_object_id();
        let image_count = self.placements.len();
        let mut page_ids = vec![];

        for placed in self.placements {
            let contents = doc.add_object(lopdf::Stream::new(dictionary! {}, vec![]));
            let page_id = doc.add_object(dictionary! {
                "Type" => "Page",
                "Parent" => pages_root,
                "MediaBox" => self.page_size.to_pdf_object_array(),
                "Contents" => lopdf::Object::Reference(contents),
            });

            let payload = placed.image.get_pdf_payload(self.lossless, self.quality);
            let pdf_image =
                lopdf::xobject::image_from(payload).expect("image payload should be decodable");
            doc.insert_image(
                page_id,
                pdf_image,
                (
                    placed.position.horizontal.points() as f32,
                    placed.position.vertical.points() as f32,
                ),
                (
                    placed.size.horizontal.points() as f32,
                    placed.size.vertical.points() as f32,
                ),
            )
            .expect("inserting image into freshly created page");
            page_ids.push(page_id.into());
        }

        doc.objects.insert(
            pages_root,
            lopdf::Object::Dictionary(dictionary! {
                "Type" => "Pages",
                "Kids" => page_ids,
                "Count" => lopdf::Object::Integer(image_count as i64),
                "MediaBox" => self.page_size.to_pdf_object_array(),
            }),
        );
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_root,
        });
        doc.trailer.set("Root", catalog_id);
        doc
    }

    pub fn new<FloatLike, PageLike>(
        _title: &str,
        page_size: PageLike,
        dpi: FloatLike,
        margin: CustomPage,
        quality: u8,
        lossless: bool,
    ) -> Self
    where
        FloatLike: Into<f64>,
        PageLike: Into<CustomPage>,
    {
        Imager {
            page_size: page_size.into(),
            dpi: dpi.into(),
            margin,
            quality,
            lossless,
            page_paths: vec![],
            placements: vec![],
        }
    }

    pub fn add_image(&mut self, image: LoadedImage) -> Result<(), ImageLoadError> {
        let page_size = self.page_size;
        let page_with_margins = page_size - self.margin;
        let image_path = image.source_path().clone();
        if page_with_margins.horizontal <= Length::zero()
            || page_with_margins.vertical <= Length::zero()
        {
            let err = ImageLoadError::MarginTooLarge {
                path: image_path.into(),
                page: page_size,
                margin: self.margin,
            };
            error!("{err}");
            return Err(err);
        }
        let adjusted_image = adjust_to_dpi(image, page_with_margins, self.dpi);

        let image_size = get_image_size(&adjusted_image, self.dpi);
        let scale = page_with_margins.fit_size(&image_size);
        let on_page = image_size * scale;
        let position = get_image_translation(page_size, on_page, self.margin);
        debug!(
            "źź AddImage scale {scale} image size {}",
            image_size.as_unit_string(pdfuse_sizing::Unit::Millimeter)
        );

        self.page_paths.push(image_path.clone());
        self.placements.push(PlacedImage {
            image: LoadedImage2::from_dynamic_image(image_path, adjusted_image),
            size: on_page,
            position,
        });
        conditional_slow_down();
        Ok(())
    }
}

/// Loads the image as a `DynamicImage` and scales it down to match the given dpi and page size.
fn adjust_to_dpi(mut image: LoadedImage, draw_area: CustomPage, dpi: f64) -> DynamicImage {
    error!(
        "Image {} of size {}x{} to fit in {} at dpi {}",
        image.source_path(),
        image.width(),
        image.height(),
        draw_area.as_unit_string(pdfuse_sizing::Unit::Millimeter),
        dpi
    );
    if dpi < 0.0 {
        error!("DPI below zero, no resizing done!",);
        return image.deconstruct().0;
    }
    // we are resizing the image, so the raw jpeg data is no longer applicable
    image.clear_raw_data();
    let horizontal_pixel_max = draw_area.horizontal.inches() * dpi;
    let vertical_pixel_max = draw_area.vertical.inches() * dpi;

    let image_width = image.width() as f64;
    let image_height = image.height() as f64;

    let scale_x = horizontal_pixel_max / image_width;
    let scale_y = vertical_pixel_max / image_height;
    let scale = scale_x.min(scale_y);
    if scale >= 1.0 {
        let target_dpi = (image.width() as f64 / draw_area.horizontal.inches()) as u32;
        debug_t!("debug.excess_dpi", dpi = target_dpi);
        return image.into();
    }

    let (dynamic_image, source_path): (DynamicImage, SourcePath) = image.deconstruct();

    debug_t!(
        "debug.resizing_image",
        name = source_path.file_name(),
        width = dynamic_image.width(),
        height = dynamic_image.height(),
        target_width = horizontal_pixel_max as u32,
        target_height = vertical_pixel_max as u32,
        scale = scale
    );
    dynamic_image.resize(
        horizontal_pixel_max as u32,
        vertical_pixel_max as u32,
        FilterType::Lanczos3,
    )
}
fn get_image_size(image: &DynamicImage, dpi: f64) -> CustomPage {
    CustomPage {
        horizontal: Length::from_inches(image.width() as f64 / dpi),
        vertical: Length::from_inches(image.height() as f64 / dpi),
    }
}
fn get_image_translation(
    page_size: CustomPage,
    image_size: CustomPage,
    margin: CustomPage,
) -> CustomPage {
    let margined_size = page_size - margin;
    // starting from bottom left (xD?)
    let difference = margined_size - image_size;
    let half_dif = difference / 2.0;
    let half_margin = margin / 2.0;
    half_dif + half_margin
}
