use pdfuse_sizing::{Unit, page::{CustomPage, Page}};
use tabled::Tabled;
use crate::displayable_range::DisplayableRange;

#[derive(Tabled)]
#[tabled(display(Option, "tabled::derive::display::option", "<none>"))]
pub struct PageInfoRow{
    #[tabled(rename = "Pages")]
    pages:DisplayableRange,
    #[tabled(rename = "Size",format("{}",self.size.as_unit_string(self.unit)))]
    size:CustomPage,
    #[tabled(rename = "Standard")]
    standard:Option<Page>,
    unit:Unit
}

