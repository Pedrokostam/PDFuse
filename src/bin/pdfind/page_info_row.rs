use crate::displayable_range::DisplayableRange;
use anyhow::Error;
use pdfuse_sizing::{
    Size, Unit,
    page::{CustomPage, Page},
};
use tabled::Tabled;

#[derive(Tabled)]
#[tabled(display(Option, "tabled::derive::display::option", "<none>"))]
pub struct PageInfoRow {
    #[tabled(rename = "Pages")]
    pub pages: DisplayableRange,
    #[tabled(rename = "Size",format("{}",self.size_or_error()))]
   pub size: Option<CustomPage>,
    #[tabled(rename = "Friendly name")]
   pub standard: Option<Page>,
    #[tabled(skip = true)]
   pub unit: Unit,
    #[tabled(rename = "Error", skip = true)]
   pub error: Option<anyhow::Error>,
}

impl PageInfoRow {
    pub fn size_or_error(&self) -> String {
        match (self.size.as_ref(), self.error.as_ref()) {
            (Some(s), _) => s.as_unit_string(self.unit),
            (None, Some(e)) => e.to_string(),
            (None, None) => panic!("That is impossible"),
        }
    }
    fn good(page: DisplayableRange, page_size: Page, unit: Option<Unit>) -> Self {
        let full_page_size: Page = page_size;
        let pages: DisplayableRange = page;
        let standard: Option<Page> = match full_page_size {
            Page::Custom(_) => None,
            _ => Some(full_page_size),
        };
        let size = Some(full_page_size.to_custom_size());
        PageInfoRow {
            pages,
            size,
            standard,
            unit: unit.unwrap_or_default(),
            error: None,
        }
    }
    fn unknown(page: DisplayableRange, e: Error) -> Self {
        PageInfoRow {
            pages: page,
            size: None,
            standard: None,
            unit: Unit::default(),
            error: Some(e),
        }
    }
    pub fn new(
        page: impl Into<DisplayableRange>,
        page_size: impl Into<anyhow::Result<Page>>,
        unit: Option<Unit>,
    ) -> Self {
        let range: DisplayableRange = page.into();
        match page_size.into() {
            Ok(size) => PageInfoRow::good(range, size, unit),
            Err(error) => PageInfoRow::unknown(range, error),
        }
    }
}
