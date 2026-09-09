use std::io::Write;

use image::{imageops::FilterType, DynamicImage};
use lopdf::{dictionary, Document};
use pdfuse_parameters::path::{SafePath, SourcePath};
use pdfuse_sizing::paper::CustomPage;
use pdfuse_sizing::{Length, Size};
use pdfuse_utils::debug_t;
use pdfuse_utils::log::{debug, error, warn};
// use printpdf::{printpdf::ImageCompression, printpdf::ImageOptimizationOptions, printpdf::PdfDocument, printpdf::PdfPage, printpdf::PdfSaveOptions, printpdf::PdfWarnMsg, printpdf::RawImage, printpdf::RawImageData, printpdf::RawImageFormat};

use crate::conditional_slow_down;
use crate::data::document_source::DocumentSources;
use crate::data::loaded_image::LoadedImage2;
use crate::data::LoadedDocument;
use crate::error::ImageLoadError;

use super::LoadedImage;

fn dynamic_to_pdf(
    image: DynamicImage,
    path: SafePath,
) -> Result<printpdf::RawImage, ImageLoadError> {
    // yoinked from printpdf
    // I couldn't find anything to create image from already loaded image.
    let width = image.width() as usize;
    let height = image.height() as usize;
    let data_format = match image.color() {
        image::ColorType::L8 => Ok(printpdf::RawImageFormat::R8),
        image::ColorType::La8 => Ok(printpdf::RawImageFormat::RG8),
        image::ColorType::Rgb8 => Ok(printpdf::RawImageFormat::RGB8),
        image::ColorType::Rgba8 => Ok(printpdf::RawImageFormat::RGBA8),
        image::ColorType::L16 => Ok(printpdf::RawImageFormat::R16),
        image::ColorType::La16 => Ok(printpdf::RawImageFormat::RG16),
        image::ColorType::Rgb16 => Ok(printpdf::RawImageFormat::RGB16),
        image::ColorType::Rgba16 => Ok(printpdf::RawImageFormat::RGBA16),
        image::ColorType::Rgb32F => Ok(printpdf::RawImageFormat::RGBF32),
        image::ColorType::Rgba32F => Ok(printpdf::RawImageFormat::RGBAF32),
        _ => Err(ImageLoadError::UnknownFormat(path.clone())),
    }?;
    let pixels = match image {
        DynamicImage::ImageLuma8(imbuffer) => Ok(printpdf::RawImageData::U8(imbuffer.into_raw())),
        DynamicImage::ImageLumaA8(imbuffer) => Ok(printpdf::RawImageData::U8(imbuffer.into_raw())),
        DynamicImage::ImageRgb8(imbuffer) => Ok(printpdf::RawImageData::U8(imbuffer.into_raw())),
        DynamicImage::ImageRgba8(imbuffer) => Ok(printpdf::RawImageData::U8(imbuffer.into_raw())),
        DynamicImage::ImageLuma16(imbuffer) => Ok(printpdf::RawImageData::U16(imbuffer.into_raw())),
        DynamicImage::ImageLumaA16(imbuffer) => {
            Ok(printpdf::RawImageData::U16(imbuffer.into_raw()))
        }
        DynamicImage::ImageRgb16(imbuffer) => Ok(printpdf::RawImageData::U16(imbuffer.into_raw())),
        DynamicImage::ImageRgba16(imbuffer) => Ok(printpdf::RawImageData::U16(imbuffer.into_raw())),
        DynamicImage::ImageRgb32F(imbuffer) => Ok(printpdf::RawImageData::F32(imbuffer.into_raw())),
        DynamicImage::ImageRgba32F(imbuffer) => {
            Ok(printpdf::RawImageData::F32(imbuffer.into_raw()))
        }
        _ => Err(ImageLoadError::UnknownPixelType(path)),
    }?;
    Ok(printpdf::RawImage {
        width,
        height,
        data_format,
        pixels,
        tag: vec![],
    })
}

pub struct Imager {
    pub(crate) document: printpdf::PdfDocument,
    pub(crate) page_size: CustomPage,
    pub(crate) dpi: f64,
    pub(crate) margin: CustomPage,
    pub(crate) pages: Vec<printpdf::PdfPage>,
    pub(crate) quality: u8,
    pub(crate) lossless: bool,
    page_paths: Vec<SourcePath>,
    pub(crate) image_data: Vec<LoadedImage2>,
}
impl Imager {
    fn get_options(&self) -> printpdf::PdfSaveOptions {
        printpdf::PdfSaveOptions {
            optimize: true,
            subset_fonts: true,
            secure: true,
            image_optimization: Some(printpdf::ImageOptimizationOptions {
                quality: Some(self.quality as f32 / 100.0),
                max_image_size: Some("2006gb".to_string()), // arbitrarily large size -> we resize the image by ourselves
                format: match self.lossless {
                    true => Some(printpdf::ImageCompression::Flate),
                    false => Some(printpdf::ImageCompression::Jpeg),
                },
                ..Default::default()
            }),
        }
    }

    pub fn close_and_into_loaded_document(self) -> LoadedDocument {
        let Imager {
            document,
            page_size,
            dpi,
            margin,
            pages,
            quality,
            lossless,
            page_paths,
            image_data,
        } = self;
        let closable_new = Imager {
            document: document.clone(),
            page_size: page_size.clone(),
            dpi: dpi.clone(),
            margin: margin.clone(),
            pages: pages.clone(),
            quality: quality.clone(),
            lossless: lossless.clone(),
            page_paths: vec![],
            image_data: image_data.clone(),
        };
        let closable = Imager {
            document,
            page_size,
            dpi,
            margin,
            pages,
            quality,
            lossless,
            page_paths: vec![],
            image_data,
        };
        let mut closed = closable_new.close_and_into_document_new();
        let _ = closed.save("new.pdf");
        let mut closed = closable.close_and_into_document();
        let _ = closed.save("old.pdf");
        let doc_sources = DocumentSources::new_multi(page_paths);
        LoadedDocument::from_document_like(doc_sources, Box::new(closed))

        // let opt = get_options();
    }

    pub fn close_and_into_document_new(self) -> lopdf::Document {
        let mut doc = lopdf::Document::new();
        let pages_root_object = doc.new_object_id();

        let mut page_ids = vec![];
        let image_count = self.image_data.len();

        for p in self.image_data {
            let img_width = Length::from_pixels(p.width(), self.dpi);
            let img_height = Length::from_pixels(p.height(), self.dpi);
            let payload = p.get_pdf_payload();
            let mut f = std::fs::File::create("AAAA.bin").unwrap();
            f.write_all(&payload).unwrap();
            let pdf_image = lopdf::xobject::image_from(payload).expect("TSETING");

            let contents = doc.add_object(lopdf::Stream::new(dictionary! {}, vec![]));
            let page_id = doc.add_object(dictionary! {
                "Type" => "Page",
                "Parent" => pages_root_object,
                "MediaBox" =>self.page_size.to_pdf_object_array(),
                "Contents" => lopdf::Object::Reference(contents)
            });

            let x_pos = (self.page_size.horizontal() - img_width) / 2.0 + self.margin.vertical();
            let y_pos = (self.page_size.vertical() - img_height) / 2.0 + self.margin.horizontal();
            println!("{x_pos} {y_pos}");
            // doc.insert_image(page_id, img_object, position, size);
            // lopdf::xobject::image_from(p)

            doc.insert_image(
                page_id,
                pdf_image,
                (x_pos.points() as f32, y_pos.points() as f32),
                (img_width.points() as f32, img_width.points() as f32),
            )
            .expect("'s oay");

            // let image_id = doc.add_object(lopdf::Stream::new(
            //     dictionary! {
            //         "Type"=>"XObject",
            //         "Subtype" => "Image",
            //         "Width" => p.width(),
            //         "Height"=>p.height(),
            //         "ColorSpace" => "DeviceRGB",
            //         "BitsPerComponent" => 8,
            //         "Filter"=>"DCTDecode",
            //     },
            //     p.into_bytes(),
            // ));
            // let content = lopdf::content::Content {
            //     operations: vec![
            //         lopdf::content::Operation::new("q", vec![]),
            //         lopdf::content::Operation::new(
            //             "cm",
            //             vec![
            //                 img_width.into(),
            //                 0.into(),
            //                 0.into(),
            //                 img_height.into(),
            //                 0.into(), // x_pos.into(),
            //                 0.into(), //                            y_pos.into(),
            //             ],
            //         ),
            //         lopdf::content::Operation::new("Do", vec!["Im1".into()]),
            //         lopdf::content::Operation::new("Q", vec![]),
            //     ],
            // };
            // let content_id = doc.add_object(lopdf::Stream::new(
            //     dictionary! {},
            //     content.encode().expect("dont worry about it"),
            // ));
            // let page_id = doc.add_object(dictionary! {
            //     "Type" => "Page",
            //     "Parent" => pages_root_object,
            //     "MediaBox" =>self.page_size.to_pdf_object_array(),
            //     "Resources" => dictionary! {
            //         "XObject" => dictionary! {
            //             "Im1" => image_id,
            //         },
            //     },
            //     "Contents" => content_id,
            // });
            page_ids.push(page_id.into());
        }
        doc.objects.insert(
            pages_root_object,
            lopdf::Object::Dictionary(dictionary! {
                "Type"=>"Pages",
                "Kids" => page_ids,
                "Count"=> lopdf::Object::Integer (image_count as i64),
                "MediaBox" =>self.page_size.to_pdf_object_array(),
            }),
        );
        let catalog_id = doc.add_object(dictionary! {
            "Type"=>"Catalog",
            "Pages"=>pages_root_object,
        });
        doc.trailer.set("Root", catalog_id);
        // doc.trailer.set("Size", (doc.objects.len() + 1) as u16);
        // doc.renumber_objects();
        doc
    }

    pub fn close_and_into_document(mut self) -> Document {
        // unsafe { self.document.get_inner() }
        let save_options = self.get_options();
        /*
        Regarding SaveOptions (for printpdf 0.8.2):
        - format
            Jpeg|Jpeg2000 -> DCTDecode
            Auto (color) -> DCTDecode
            Auto (gray) -> FlateDecode
            AllElse -> FlateDecode

            Alpha is encoded separately (usually flate) and applied as a mask
        - quality
            Only DCTDecode uses quality (which should be (0,1> as it is multiplied by 100 in crate)

        - max_image_size
            If uncompressed(!) image would exceed the size, scale it down

         */
        let mut warnings: Vec<printpdf::PdfWarnMsg> = vec![];
        let bytes = self
            .document
            .with_pages(self.pages)
            .save(&save_options, &mut warnings);
        for w in warnings {
            warn!("Warning {}: {}", w.page, w.msg);
        }
        Document::load_mem(&bytes).unwrap()
    }

    pub fn new<FloatLike, PageLike>(
        title: &str,
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
            document: printpdf::PdfDocument::new(title),
            page_size: page_size.into(),
            dpi: dpi.into(),
            margin,
            pages: vec![],
            quality,
            lossless,
            page_paths: vec![],
            image_data: vec![],
        }
    }

    pub fn add_image(&mut self, image: LoadedImage) -> Result<(), ImageLoadError> {
        let page_size = self.page_size;
        let page_with_margins = page_size - self.margin;
        let image_path = image.source_path().clone();
        let adjusted_image = adjust_to_dpi(image, page_with_margins, self.dpi);

        let image_size = get_image_size(&adjusted_image, self.dpi);

        self.image_data.push(LoadedImage2::from_dynamic_image(
            image_path.clone(),
            adjusted_image.clone(),
        ));
        let pdf_image = dynamic_to_pdf(adjusted_image, image_path.clone().into())?;

        let image_id = self.document.add_image(&pdf_image);
        let scale = page_with_margins.fit_size(&image_size);
        let translation = get_image_translation(page_size, image_size * scale, self.margin);
        debug!(
            "źź AddImage scale {scale} image size {}",
            image_size.as_unit_string(pdfuse_sizing::Unit::Millimeter)
        );
        let image_contents = printpdf::Op::UseXobject {
            id: image_id,
            transform: printpdf::XObjectTransform {
                scale_x: Some(scale as f32),
                scale_y: Some(scale as f32),
                dpi: Some(self.dpi as f32),
                translate_x: Some(printpdf::Pt(translation.vertical.points() as f32)),
                translate_y: Some(printpdf::Pt(translation.vertical.points() as f32)),
                rotate: None,
            },
        };
        let page = printpdf::PdfPage::new(
            printpdf::Pt(page_size.horizontal.points() as f32).into(),
            printpdf::Pt(page_size.vertical.points() as f32).into(),
            vec![image_contents],
        );
        self.page_paths.push(image_path);
        self.pages.push(page);
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
