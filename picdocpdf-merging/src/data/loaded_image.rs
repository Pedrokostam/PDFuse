use std::path::{Path, PathBuf};

use image::{DynamicImage, ImageReader};


pub struct LoadedImage{
    image:DynamicImage,
    source_path:PathBuf
}
impl From<LoadedImage> for DynamicImage {
    fn from(value: LoadedImage) -> Self {
        value.image
    }
}
impl LoadedImage {
    pub fn width(&self)->u32{
        self.image.width()
    }
    pub fn height(&self)->u32{
        self.image.height()
    }
    pub fn source_path(&self)->&Path{
        self.source_path.as_path()
    }
    pub fn load(path: impl AsRef<Path>) -> std::io::Result<LoadedImage> {
        let image_reader = ImageReader::open(path.as_ref()).and_then(|r| r.with_guessed_format())?;
        let decoded_image = image_reader
            .decode()
            .expect("Format should be already successfully detected");
        Ok(LoadedImage{image:decoded_image,source_path:path.as_ref().to_path_buf()})
    }
}