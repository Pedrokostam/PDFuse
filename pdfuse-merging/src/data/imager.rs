
use std::time::Instant;

use image::{imageops::FilterType, DynamicImage};
use lopdf::Document;
use pdfuse_parameters::SafePath;
use pdfuse_sizing::{CustomSize, Length, Size};
use pdfuse_utils::debug_t;
use pdfuse_utils::log::debug;
// use lopdf::Document, Image, ImageTransform, ImageXObject, PdfDocumentReference, PdfLayerReference,
use printpdf::ImageCompression;
use printpdf::{
    ImageOptimizationOptions, PdfDocument, PdfPage, RawImageData, RawImageFormat,
};
use printpdf::{PdfSaveOptions, PdfWarnMsg, RawImage};

use crate::conditional_slow_down;
use crate::error::ImageLoadError;

use super::LoadedImage;

fn dynamic_to_pdf(image: DynamicImage,path:SafePath) -> Result<RawImage, ImageLoadError> {

    // yoinked from printpdf
    // I couldn't find anything to create image from already loaded image.
    let width = image.width() as usize;
    let height = image.height() as usize;
    let data_format = match image.color() {
        image::ColorType::L8 => Ok(RawImageFormat::R8),
        image::ColorType::La8 => Ok(RawImageFormat::RG8),
        image::ColorType::Rgb8 => Ok(RawImageFormat::RGB8),
        image::ColorType::Rgba8 => Ok(RawImageFormat::RGBA8),
        image::ColorType::L16 => Ok(RawImageFormat::R16),
        image::ColorType::La16 => Ok(RawImageFormat::RG16),
        image::ColorType::Rgb16 => Ok(RawImageFormat::RGB16),
        image::ColorType::Rgba16 => Ok(RawImageFormat::RGBA16),
        image::ColorType::Rgb32F => Ok(RawImageFormat::RGBF32),
        image::ColorType::Rgba32F => Ok(RawImageFormat::RGBAF32),
        _ => Err(ImageLoadError::UnknownFormat(path.clone())),
    }?;
    let pixels = match image {
        DynamicImage::ImageLuma8(imbuffer) => Ok(RawImageData::U8(imbuffer.into_raw())),
        DynamicImage::ImageLumaA8(imbuffer) => Ok(RawImageData::U8(imbuffer.into_raw())),
        DynamicImage::ImageRgb8(imbuffer) => Ok(RawImageData::U8(imbuffer.into_raw())),
        DynamicImage::ImageRgba8(imbuffer) => Ok(RawImageData::U8(imbuffer.into_raw())),
        DynamicImage::ImageLuma16(imbuffer) => Ok(RawImageData::U16(imbuffer.into_raw())),
        DynamicImage::ImageLumaA16(imbuffer) => Ok(RawImageData::U16(imbuffer.into_raw())),
        DynamicImage::ImageRgb16(imbuffer) => Ok(RawImageData::U16(imbuffer.into_raw())),
        DynamicImage::ImageRgba16(imbuffer) => Ok(RawImageData::U16(imbuffer.into_raw())),
        DynamicImage::ImageRgb32F(imbuffer) => Ok(RawImageData::F32(imbuffer.into_raw())),
        DynamicImage::ImageRgba32F(imbuffer) => Ok(RawImageData::F32(imbuffer.into_raw())),
        _ => Err(ImageLoadError::UnknownPixelType(path)),
    }?;
    // debug!("Data format: {data_format:?}");
    Ok(RawImage {
        width,
        height,
        data_format,
        pixels,
        tag: vec![],
    })
}

pub struct Imager {
    pub(crate) document: PdfDocument,
    pub(crate) page_size: CustomSize,
    pub(crate) dpi: f64,
    pub(crate) margin: CustomSize,
    pub(crate) pages: Vec<PdfPage>,
    pub(crate) quality: u8,
    pub(crate) lossless: bool,
}
impl Imager {
    pub fn close_and_into_document(mut self) -> Document {
        // unsafe { self.document.get_inner() }
        let save_options = PdfSaveOptions {
            optimize: true,
            subset_fonts: true,
            secure: true,
            image_optimization: Some(ImageOptimizationOptions {
                quality: Some(self.quality as f32 / 100.0),
                max_image_size: Some("2137gb".to_string()), // "arbitrarily" large size -> we resize the image by ourselves
                format:  match self.lossless{
                    true =>Some(ImageCompression::Flate),
                    false => Some(ImageCompression::Jpeg),
                },
                ..Default::default()
            }),
        };
        /*
        Regarding SaveOptions (for printpdf 0.8.2):
        - format
            Jpeg|Jpeg2000 -> DCTDecode
            Auto (color) -> DCTDeoced
            Auto (gray) -> FlateDecode
            AllElse -> FlateDecode

            Alpha is encoded separately (usually flate) and applied as a mask
        - quality
            Only DCTDecode uses quality (which should be (0,1> as it is multiplied by 100 in crate)

        - max_image_size
            If uncompressed(!) image would exceed the size, scale it down

         */
        let mut warnings: Vec<PdfWarnMsg> = vec![];
        let bytes = self
            .document
            .with_pages(self.pages)
            .save(&save_options, &mut warnings);
         Document::load_mem(&bytes).unwrap()
    }
    pub fn new<FloatLike, PageLike>(
        title: &str,
        page_size: PageLike,
        dpi: FloatLike,
        margin: CustomSize,
        quality: u8,
        lossless: bool,
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
            quality,
            lossless,
        }
    }

    pub fn add_image(&mut self, image: LoadedImage) -> Result<(), ImageLoadError> {
        let page_size = self.page_size;
        let page_with_margins = page_size - self.margin;
        let image_path= image.source_path().clone();
        let adjusted_image = adjust_to_dpi(image, page_with_margins, self.dpi);

        let image_size = get_image_size(&adjusted_image, self.dpi);

        let pdf_image = dynamic_to_pdf(adjusted_image,image_path)?;
        
        let image_id = self.document.add_image(&pdf_image);
        let scale = page_with_margins.fit_size(&image_size);
        let translation = get_image_translation(page_size, image_size * scale, self.margin);
        // debug!("{scale}");
        let image_contents = printpdf::Op::UseXobject {
            id: image_id,
            transform: printpdf::XObjectTransform {
                scale_x: Some(scale as f32),
                scale_y: Some(scale as f32),
                dpi: Some(self.dpi as f32),
                translate_x: Some(translation.horizontal.into()),
                translate_y: Some(translation.vertical.into()),
                rotate: None,
            },
        };
        let page = PdfPage::new(
            page_size.horizontal.into(),
            page_size.vertical.into(),
            vec![image_contents],
        );
        self.pages.push(page);
        conditional_slow_down();
        Ok(())
    }
}

fn adjust_to_dpi(image: LoadedImage, draw_area: CustomSize, dpi: f64) -> DynamicImage {
    let horizontal_pixel_max = draw_area.horizontal.inch() * dpi;
    let vertical_pixel_max = draw_area.vertical.inch() * dpi;
    let image_width = image.width() as f64;
    let image_height = image.height() as f64;
    let scale_x = horizontal_pixel_max / image_width;
    let scale_y = vertical_pixel_max / image_height;
    let scale = scale_x.min(scale_y);
    if scale >= 1.0 {
        let target_dpi = (image.width() as f64 / draw_area.horizontal.inch()) as u32;
        // debug_t!("debug.excess_dpi", dpi = target_dpi);
        return image.into();
    }
    let (dynamic_image, safe_path): (DynamicImage, SafePath) = image.deconstruct();
    // debug_t!(
    //     "debug.resizing_image",
    //     name = safe_path.file_name().unwrap().to_string_lossy(),
    //     width = dynamic_image.width(),
    //     height = dynamic_image.height(),
    //     target_width = horizontal_pixel_max as u32,
    //     target_height = vertical_pixel_max as u32,
    //     scale = scale
    // );
    debug!{"{safe_path} page: {draw_area}"};
    dynamic_image.resize(
        horizontal_pixel_max as u32,
        vertical_pixel_max as u32,
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
