#![allow(dead_code)]
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Cursor, Read, Seek},
    path::{Path, PathBuf},
};

use image::{DynamicImage, ImageReader};
use pdfuse_parameters::path::{SafePath, SourcePath};
use thiserror::Error;

use crate::{conditional_slow_down, error::ImageLoadError};

/// An enumeration holding either a `DynamicImage` (if the image was created or modified)
/// or the raw file data (if it hasn't been touched)
#[derive(Debug, Clone)]
enum ImageData {
    // NoData,
    Raw {
        file_data: Vec<u8>,
        width: u32,
        height: u32,
        format: image::ImageFormat,
    },
    Dynamic(DynamicImage),
}

#[derive(Error, Debug)]
pub enum ImageReadingError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Decode(#[from] image::ImageError),
}

#[derive(Clone, Debug)]
pub struct LoadedImage2 {
    source_path: SourcePath,
    data: ImageData,
}

impl LoadedImage2 {
    pub fn width(&self) -> u32 {
        match &self.data {
            ImageData::Raw {
                file_data: _,
                width,
                height: _,
                format: _,
            } => *width,
            ImageData::Dynamic(dynamic_image) => dynamic_image.width(),
        }
    }
    pub fn height(&self) -> u32 {
        match &self.data {
            ImageData::Raw {
                file_data: _,
                width: _,
                height,
                format: _,
            } => *height,
            ImageData::Dynamic(dynamic_image) => dynamic_image.height(),
        }
    }
    pub fn source_path(&self) -> &SourcePath {
        &self.source_path
    }

    fn get_reader<B>(stream: B) -> Result<ImageReader<B>, std::io::Error>
    where
        B: BufRead + Seek,
    {
        image::ImageReader::new(stream).with_guessed_format()
    }

    fn load_impl<B>(stream: B, source_path: SourcePath) -> Result<Self, ImageReadingError>
    where
        B: BufRead + Seek,
    {
        let reader = Self::get_reader(stream)?;
        if reader.format() == Some(image::ImageFormat::Jpeg) {
            let format = reader.format().expect("Already checked");
            let (width, height) = reader
                .into_dimensions()
                .expect("Already checked that format is OK");
            let mut file_data = vec![];
            let _ = std::fs::File::open(&source_path)?.read_to_end(&mut file_data)?;
            return Ok(Self {
                source_path,
                data: ImageData::Raw {
                    file_data,
                    width,
                    height,
                    format,
                },
            });
        }
        let dyn_data = reader.decode()?;
        let data = ImageData::Dynamic(dyn_data);
        Ok(Self { source_path, data })
    }

    pub fn load_file(path: impl AsRef<Path>) -> Result<Self, ImageReadingError> {
        let source_path = SourcePath::Image(path.as_ref().into());
        let file_handle = std::fs::File::open(&source_path)?;
        let file_reader = BufReader::new(file_handle);
        Self::load_impl(file_reader, source_path)
    }

    pub fn load_from<B>(stream: B, path: impl AsRef<Path>) -> Result<Self, ImageReadingError>
    where
        B: BufRead + Seek,
    {
        let source_path = SourcePath::Image(path.as_ref().into());
        Self::load_impl(stream, source_path)
    }

    pub fn load_from_slice(
        slice: &[u8],
        path: impl AsRef<Path>,
    ) -> Result<Self, ImageReadingError> {
        let source_path = SourcePath::Image(path.as_ref().into());
        let cursor = Cursor::new(slice.to_vec());
        let reader = BufReader::new(cursor);
        Self::load_impl(reader, source_path)
    }

    pub fn get_pdf_payload(self) -> Vec<u8> {
        match self.data {
            ImageData::Raw {
                file_data,
                width: _,
                height: _,
                format: _,
            } => file_data,
            ImageData::Dynamic(dynamic_image) => {
                let payload = Vec::with_capacity(3 * 1024 * 1024);
                let cursor = Cursor::new(payload);
                let mut writer = BufWriter::new(cursor);
                let _ = dynamic_image.write_to(&mut writer, image::ImageFormat::Bmp);
                writer
                    .into_inner()
                    .expect("Encoding to BMP should not fail")
                    .into_inner()
            }
        }
    }

    /// Ensures the image data has the form of `DynamicImage` and can be further processed,
    /// returning a reference to the dynamic image.
    ///
    /// It mutates the object - it will always contain dynamic image from now on
    ///
    /// # Errors
    ///
    /// This function will return an error if it cannot decode the raw data to an image.
    pub fn prepare_for_processing(&mut self) -> Result<&mut DynamicImage, ImageReadingError> {
        let current = std::mem::replace(
            &mut self.data,
            ImageData::Raw {
                file_data: vec![],
                width: 0,
                height: 0,
                format: image::ImageFormat::Bmp,
            },
        );
        match current {
            ImageData::Raw {
                file_data,
                width,
                height,
                format,
            } => {
                let cursor = Cursor::new(file_data);
                let reader = BufReader::new(cursor);
                let decoder = image::ImageReader::with_format(reader, format);
                let decoded_data = decoder.decode()?;
                assert!(
                    width == decoded_data.width(),
                    "Decoded and declared widths are different (path: {})",
                    self.source_path
                );
                assert!(
                    height == decoded_data.height(),
                    "Decoded and declared heights are different (path: {})",
                    self.source_path
                );
                self.data = ImageData::Dynamic(decoded_data);
            }
            ImageData::Dynamic(dynamic_image) => {
                self.data = ImageData::Dynamic(dynamic_image);
            }
        };
        match &mut self.data {
            ImageData::Dynamic(dynamic_image) => Ok(dynamic_image),
            _ => unreachable!(),
        }
    }
    pub fn from_dynamic_image(source_path: impl AsRef<Path>, img: DynamicImage) -> Self {
        Self {
            source_path: SourcePath::Image(source_path.as_ref().into()),
            data: ImageData::Dynamic(img),
        }
    }
}

pub struct LoadedImage {
    image: Box<DynamicImage>,
    raw_data: Option<Vec<u8>>,
    source_path: SourcePath,
}
impl From<LoadedImage> for DynamicImage {
    fn from(value: LoadedImage) -> Self {
        value.deconstruct().0
    }
}
impl LoadedImage {
    pub fn width(&self) -> u32 {
        self.image.width()
    }
    pub fn deconstruct(self) -> (DynamicImage, SourcePath) {
        (*self.image, self.source_path)
    }
    pub fn height(&self) -> u32 {
        self.image.height()
    }
    pub fn source_path(&self) -> &SourcePath {
        &self.source_path
    }
    pub fn clear_raw_data(&mut self) {
        self.raw_data = None;
    }
    pub fn to_smallest_raw(self) -> Vec<u8> {
        if let Some(vec) = self.raw_data {
            return vec;
        }
        self.image.into_bytes()
    }
    pub fn load(path: impl AsRef<Path>) -> Result<LoadedImage, ImageLoadError> {
        let source_path = SourcePath::Image(path.as_ref().into());
        let file = File::open(&source_path)?;

        // kinda bad, but it lets as keeps the original data
        let file_len = file.metadata()?.len() as usize;
        let mut reader = BufReader::new(file);
        let mut raw_data_vec = Vec::with_capacity(file_len);

        reader.read_to_end(&mut raw_data_vec)?;
        // seek to beginning to let image_reader to do the rest
        reader.seek(std::io::SeekFrom::Start(0))?;
        // TODO turns out the recognition may fail, huh
        let image_reader = ImageReader::new(reader)
            .with_guessed_format()
            .map_err(|_| ImageLoadError::UnknownFormat(SafePath::from(path.as_ref())))?;

        // we need the raw jpeg data if we do not resize the image (avoid recompressing as deflate)
        let raw_data = if image_reader.format() == Some(image::ImageFormat::Jpeg) {
            Some(raw_data_vec)
        } else {
            None
        };

        // we need decoded image to get things such as size, or to resize it later
        let decoded_image = image_reader
            .decode()
            .map_err(|_| ImageLoadError::UnknownFormat(SafePath::from(path.as_ref())))?;
        conditional_slow_down();
        Ok(LoadedImage {
            image: Box::new(decoded_image),
            raw_data,
            source_path,
        })
    }
}
