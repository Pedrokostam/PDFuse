use image::{imageops::FilterType, DynamicImage};
use pdfuse_sizing::{CustomSize, Length, Size};
use pdfuse_utils::debug_t;
use printpdf::{
    lopdf::Document, Image, ImageTransform, ImageXObject, PdfDocumentReference, PdfLayerReference,
    Px,
};

pub struct Imager {
    pub(crate) document: PdfDocumentReference,
    pub(crate) page_size: CustomSize,
    pub(crate) dpi: f64,
    pub(crate) margin: CustomSize,
    page_count: usize,
}
impl Imager {
    pub fn close_and_into_document(self) -> Document {
        // unsafe { self.document.get_inner() }
        let bytes = self.document.save_to_bytes().unwrap();
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
            document: printpdf::PdfDocument::empty(title)
                .with_conformance(printpdf::PdfConformance::A2_2011_PDF_1_7),
            page_size: page_size.into(),
            dpi: dpi.into(),
            margin,
            page_count: 0,
        }
    }

    fn add_page_for_image(&mut self) -> PdfLayerReference {
        let (page_index, layer_index) = self.document.add_page(
            self.page_size.horizontal.into(),
            self.page_size.vertical.into(),
            "Image",
        );
        self.document.get_page(page_index).get_layer(layer_index)
    }

    pub fn page_count(&self) -> usize {
        self.page_count
    }

    pub fn add_image(&mut self, image: impl Into<DynamicImage>) {
        let page_size = self.page_size;
        let decoded_image = image.into();
        let page_with_margins = page_size - self.margin;

        let adjusted_image = adjust_to_dpi(decoded_image, page_with_margins, self.dpi);

        let image_size = get_image_size(&adjusted_image, self.dpi);

        let pdf_image: ImageXObject = printpdf::ImageXObject {
            width: Px(adjusted_image.width() as usize),
            height: Px(adjusted_image.height() as usize),
            color_space: printpdf::ColorSpace::Rgb,
            bits_per_component: printpdf::ColorBits::Bit8,
            interpolate: true,
            image_data: adjusted_image.to_rgb8().into_vec(),
            image_filter: None,
            smask: None,
            clipping_bbox: None,
        };
        let current_layer = self.add_page_for_image();

        let scale = page_with_margins.fit_size(&image_size);
        let translation = get_image_translation(page_size, image_size * scale, self.margin);

        // scaling -> rotation -> translation
        let transform = ImageTransform {
            scale_x: Some(scale as f32),
            scale_y: Some(scale as f32),
            dpi: Some(self.dpi as f32),
            translate_x: Some(translation.horizontal.into()),
            translate_y: Some(translation.vertical.into()),
            rotate: None,
        };
        let image_to_add: Image = pdf_image.into();
        image_to_add.add_to_layer(current_layer, transform);
        self.page_count += 1;
    }
}

fn adjust_to_dpi(image: DynamicImage, draw_area: CustomSize, dpi: f64) -> DynamicImage {
    let horizontal_pixel_max = (draw_area.horizontal.inch() * dpi) as u32;
    let vertical_pixel_max = (draw_area.vertical.inch() * dpi) as u32;
    if horizontal_pixel_max > image.width() || vertical_pixel_max > image.height() {
        let target_dpi = (image.width() as f64 / draw_area.horizontal.inch()) as u32;
        debug_t!("debug.excess_dpi", dpi = target_dpi);
        return image;
    }
    debug_t!(
        "debug.resizing_image",
        width = image.width(),
        height = image.height(),
        target_width = horizontal_pixel_max,
        target_height = vertical_pixel_max
    );
    image.resize(
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
