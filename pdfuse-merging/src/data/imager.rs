use image::{imageops::FilterType, DynamicImage};
use lopdf::Document;
use pdfuse_sizing::{CustomSize, Length, Size, Unit};
use pdfuse_utils::{
    debug_t,
    log::{self, logger},
};
// use lopdf::Document, Image, ImageTransform, ImageXObject, PdfDocumentReference, PdfLayerReference,
use printpdf::{ImageOptimizationOptions, Layer, PdfDocument,PdfPage};
use printpdf::{PdfSaveOptions, PdfWarnMsg, RawImage};
use  printpdf::units::{Pt,Mm};

use super::LoadedImage;

pub struct Imager {
    pub(crate) document: PdfDocument,
    pub(crate) page_size: CustomSize,
    pub(crate) dpi: f64,
    pub(crate) margin: CustomSize,
    pub(crate) pages: Vec<PdfPage>,
}
impl Imager {
    pub fn close_and_into_document(mut self) -> Document {
        // unsafe { self.document.get_inner() }
        let save_options = PdfSaveOptions {
            optimize: true,
            subset_fonts: true,
            secure: true,
            image_optimization: Some(ImageOptimizationOptions {
                ..Default::default()
            }),
        };
        let mut warnings: Vec<PdfWarnMsg> = vec![];
        let bytes = self.document.with_pages(self.pages).save(&save_options, &mut warnings);
        Document::load_mem(&bytes).unwrap()
    }
    pub fn new<FloatLike, PageLike>(
        title: &str,
        page_size: PageLike,
        dpi: FloatLike,
        margin: CustomSize,
    ) -> Self
    where
        FloatLike: Into<f64>,
        PageLike: Into<CustomSize>,
    {
        Imager {
            document: printpdf::PdfDocument::new(title),
            page_size: page_size.into(),
            dpi: dpi.into(),
            margin,
            pages: vec![],
        }
    }

    // fn add_page_for_image(&mut self) -> Layer {
    //     let pp =  PdfPage::new(
    //         self.page_size.horizontal.into(),
    //         self.page_size.vertical.into(),
    //         vec![]);
    //     self.document.pages.push( pp);
    //     self.document.pages.last().unwrap().get_layers()[0]
    // }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn add_image(&mut self, image: LoadedImage) {
        let page_size = self.page_size;
        let decoded_image = image.into();
        let page_with_margins = page_size - self.margin;

        let mut adjusted_image = adjust_to_dpi(decoded_image, page_with_margins, self.dpi);

        let image_size = get_image_size(&adjusted_image, self.dpi);
        let mut warnings: Vec<PdfWarnMsg> = vec![];
        let warned_image =
            RawImage::decode_from_bytes(adjusted_image.as_mut_rgb8().unwrap(), &mut warnings);
        let pdf_image: RawImage;
        match warned_image {
            Ok(iw) => pdf_image = iw,
            Err(e) => {
                log::error!("{e}");
                return;
            }
        }
        let image_id =self.document.add_image(&pdf_image);
        let scale = page_with_margins.fit_size(&image_size);
        let translation = get_image_translation(page_size, image_size * scale, self.margin);
        let image_contents = vec![
            printpdf::Op::UseXobject { id: image_id, transform:printpdf::XObjectTransform {
            scale_x: Some(scale as f32),
            scale_y: Some(scale as f32),
            dpi: Some(self.dpi as f32),
            translate_x: Some(translation.horizontal.into()),
            translate_y: Some(translation.vertical.into()),
            rotate: None,
        } }
        ];
        let page = PdfPage::new(page_size.horizontal.into(), page_size.vertical.into(), image_contents);
        self.pages.push(page);
    }
}

fn adjust_to_dpi(image: LoadedImage, draw_area: CustomSize, dpi: f64) -> DynamicImage {
    let horizontal_pixel_max = (draw_area.horizontal.inch() * dpi) as u32;
    let vertical_pixel_max = (draw_area.vertical.inch() * dpi) as u32;
    if horizontal_pixel_max > image.width() || vertical_pixel_max > image.height() {
        let target_dpi = (image.width() as f64 / draw_area.horizontal.inch()) as u32;
        debug_t!("debug.excess_dpi", dpi = target_dpi);
        return image.into();
    }
    debug_t!(
        "debug.resizing_image",
        name = image.source_path().file_name().unwrap().to_string_lossy(),
        width = image.width(),
        height = image.height(),
        target_width = horizontal_pixel_max,
        target_height = vertical_pixel_max
    );
    image.to_dynamic_image().resize(
        horizontal_pixel_max,
        vertical_pixel_max,
        FilterType::Lanczos3,
    )
}
fn get_image_size(image: &DynamicImage, dpi: f64) -> CustomSize {
    CustomSize {
        horizontal: Length::from_inches(image.width() as f64 / dpi),
        vertical: Length::from_inches(image.height() as f64 / dpi),
    }
}
fn get_image_translation(
    page_size: CustomSize,
    image_size: CustomSize,
    margin: CustomSize,
) -> CustomSize {
    let margined_size = page_size - margin;
    // starting from bottom left (xD?)
    let difference = margined_size - image_size;
    let half_dif = difference / 2.0;
    let half_margin = margin / 2.0;
    half_dif + half_margin
}
