use crate::page_info_row::PageInfoRow;


pub(crate) struct Bundler {
    pub pages: Vec<usize>,
    pub value: PageInfoRow,
}
impl Bundler {
    pub fn new(piw: PageInfoRow) -> Self {
        Bundler {
            pages: piw.pages.clone().into_vec(),
            value: piw,
        }
    }
}
impl From<Bundler> for PageInfoRow{
    fn from(value: Bundler) -> Self {
        PageInfoRow::new(value.pages, value.value.standard)
    }
}
