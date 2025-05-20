
use pdfuse_parameters::{Parameters, SourcePath};
use pdfuse_sizing::{CustomSize, Size};
use pdfuse_utils::Indexed;

use super::{Data, PdfResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GuideRequirement {
    /// Two cases:
    /// <ul>
    ///     <li>There are only images and/or PDFs.</li>
    ///     <li>Fallback size is forced.</li>
    /// </ul>
    /// PDFs are ignored, since it easy to get their size.
    SizeInformationNotNeeded,
    /// There are libre documents <b>before</b> images and fallback size is not forced.
    WaitForLibreConversion,
    /// There are libre docuements, but their size will not affect images. Two cases:
    /// <ul>
    ///     <li>No libre documents are before images.</li>
    ///     <li>Fallback size is forced.</li>
    /// </ul>
    RunInParallelWithLibreConversion,
}
#[derive(Clone, Debug)]
pub(crate) struct SizeGuide {
    map: Vec<CustomSize>,
    fallback: CustomSize,
}
impl SizeGuide {
    pub fn need_to_wait_for_pdf_threads(
        source_paths: &[Indexed<SourcePath>],
        parameters: &Parameters,
    ) -> GuideRequirement {
        if parameters.force_image_page_fallback_size {
            return GuideRequirement::SizeInformationNotNeeded;
        }
        let mut document_before_image = false;
        let mut has_any_image = false;
        let mut has_any_document = false;
        let mut is_previous_element_document = false;
        for path in source_paths.iter().map(|p| p.value()) {
            match path {
                SourcePath::Image(_) => {
                    has_any_image = true;
                    document_before_image |= is_previous_element_document;
                    is_previous_element_document = false;
                }
                SourcePath::Pdf(_) => {
                    // skip
                    is_previous_element_document = false;
                }
                SourcePath::LibreDocument(_) => {
                    is_previous_element_document = true;
                    has_any_document = true;
                }
            }
        }
        println! {"{has_any_image:?},{document_before_image:?}"};
        match (has_any_image, has_any_document, document_before_image) {
            // No images, no worry
            (false, _, _) => GuideRequirement::SizeInformationNotNeeded,
            // No documents, no worry
            (true, false, _) => GuideRequirement::SizeInformationNotNeeded,
            // Docs before images
            (true, true, true) => GuideRequirement::WaitForLibreConversion,
            // Docs, but not affecting
            (true, _, false) => GuideRequirement::RunInParallelWithLibreConversion,
        }
    }

    /// Creates a new [`SizeGuide`].\
    /// all_data: slice with all pdfs (converted beforehand or not) that will be needed for the guide.
    pub fn new(all_data: &[Indexed<PdfResult<Data>>], parameters: &Parameters) -> Self {
        let fallback = parameters.image_page_fallback_size.to_custom_size();
        if parameters.force_image_page_fallback_size {
            return SizeGuide {
                map: Default::default(),
                fallback,
            };
        }
        let mut max_index: usize = 0;

        if let Some(max_item) = all_data.iter().max_by_key(|x| x.index()) {
            max_index = max_index.max(max_item.index());
        }
        max_index += 1;
        let mut proto_guide: Vec<CustomSize> = std::iter::repeat_n(fallback, max_index).collect();

        let mut last_index = 0;
        let mut last_size = fallback;
        for (index, custom_size) in all_data.iter().filter_map(|x| match x.value() {
            Ok(data) => match data {
                Data::Document(loaded_document) => {
                    Some((x.index(), loaded_document.page_size().unwrap_or(fallback)))
                }
                _ => None,
            },
            Err(_) => None,
        }) {
            proto_guide[last_index..index].fill(last_size);
            last_index = index;
            last_size = custom_size;
        }
        proto_guide[last_index..max_index].fill(last_size);
        SizeGuide {
            map: Default::default(),
            fallback,
        }
    }

    pub fn get_size(&self, index: usize) -> CustomSize {
        *self.map.get(index).unwrap_or(&self.fallback)
    }
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use super::*;

    fn indexise(paths: &[SourcePath]) -> Vec<Indexed<SourcePath>> {
        paths
            .iter()
            .enumerate()
            .map(|(index, value)| Indexed::new(index, value.clone()))
            .collect()
    }
    fn pdf() -> SourcePath {
        SourcePath::Pdf(PathBuf::new())
    }
    fn image() -> SourcePath {
        SourcePath::Image(PathBuf::new())
    }
    fn libre() -> SourcePath {
        SourcePath::LibreDocument(PathBuf::new())
    }
    fn params() -> Parameters {
        Parameters {
            force_image_page_fallback_size: false,
            ..Default::default()
        }
    }

    #[test]
    fn test_can_create_guide_only_images() {
        let source_paths = indexise(&[image(), image(), image()]);
        assert_eq!(
            SizeGuide::need_to_wait_for_pdf_threads(&source_paths, &params()),
            GuideRequirement::SizeInformationNotNeeded
        )
    }

    #[test]
    fn test_can_create_guide_only_images_and_pdfs() {
        let source_paths = indexise(&[pdf(), image(), image(), image()]);
        assert_eq!(
            SizeGuide::need_to_wait_for_pdf_threads(&source_paths, &params()),
            GuideRequirement::SizeInformationNotNeeded
        )
    }

    #[test]
    fn test_can_create_guide_docs_after_images() {
        let source_paths = indexise(&[libre(), image(), image(), image()]);
        assert_eq!(
            SizeGuide::need_to_wait_for_pdf_threads(&source_paths, &params()),
            GuideRequirement::WaitForLibreConversion
        )
    }

    #[test]
    fn test_can_create_guide_docs_after_images_forced_fallback() {
        let source_paths = indexise(&[libre(), image(), image(), image()]);
        assert_eq!(
            SizeGuide::need_to_wait_for_pdf_threads(&source_paths,&Parameters { force_image_page_fallback_size:true,..Default::default()} ),
            GuideRequirement::SizeInformationNotNeeded
        )
    }

    #[test]
    fn test_can_create_guide_pdf_after_images() {
        let source_paths = indexise(&[image(), image(), image(), libre()]);
        assert_eq!(
            SizeGuide::need_to_wait_for_pdf_threads(&source_paths, &params()),
            GuideRequirement::RunInParallelWithLibreConversion
        )
    }
}
