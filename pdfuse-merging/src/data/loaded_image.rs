use std::path::Path;

use image::{DynamicImage, ImageReader};
use pdfuse_parameters::SafePath;

use crate::{conditional_slow_down, error::ImageLoadError};

pub struct LoadedImage {
    image: Box<DynamicImage>,
    source_path: SafePath,
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
    pub fn deconstruct(self) -> (DynamicImage, SafePath) {
        (*self.image, self.source_path)
    }
    pub fn height(&self) -> u32 {
        self.image.height()
    }
    pub fn source_path(&self) -> &SafePath {
        &self.source_path
    }
    pub fn load(path: impl AsRef<Path>) -> Result<LoadedImage, ImageLoadError> {
        // TODO turns out the recognition may fail, huh
        let image_reader = ImageReader::open(path.as_ref())
            .and_then(|r| r.with_guessed_format())
            .map_err(|_| ImageLoadError::UnknownFormat(SafePath::from(path.as_ref())))?;
        let decoded_image = image_reader
            .decode()
            .map_err(|_| ImageLoadError::UnknownFormat(SafePath::from(path.as_ref())))?;
        conditional_slow_down();
        Ok(LoadedImage {
            image: Box::new(decoded_image),
            source_path: SafePath::new(path),
        })
    }
}
