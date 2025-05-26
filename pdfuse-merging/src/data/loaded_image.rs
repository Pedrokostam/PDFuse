use std::path::{Path};

use image::{DynamicImage, ImageReader};
use pdfuse_parameters::SafePath;

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
    pub fn load(path: impl AsRef<Path>) -> std::io::Result<LoadedImage> {
        let image_reader =
            ImageReader::open(path.as_ref()).and_then(|r| r.with_guessed_format())?;
        let decoded_image = image_reader
            .decode()
            .expect("Format should be already successfully detected");
        Ok(LoadedImage {
            image: Box::new(decoded_image),
            source_path: SafePath::new(path),
        })
    }
}
