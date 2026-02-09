use crate::{bundler::Bundler, displayable_range::DisplayableRange};
use pdfuse_sizing::{
    Size, Unit,
    paper::Page,
};
use tabled::builder::Builder;
#[derive(Debug)]
pub struct PageInfoRow {
    pub pages: DisplayableRange,
    // #[tabled(rename = "Size",format("{}",self.size_or_error()))]
    // pub size: Option<CustomPage>,
    // #[tabled(rename = "Friendly name", format("{}",self.standard_name()))]
    pub standard: anyhow::Result<Page>,
    // #[tabled(skip = true)]
    // pub unit: Unit,
    // #[tabled(rename = "Error", skip = true)]
    // pub error: Option<anyhow::Error>,
    pub(crate) pseudo_hash: String,
}


impl PageInfoRow {
    pub fn bundle_data(items: impl IntoIterator<Item = PageInfoRow>) -> Vec<PageInfoRow> {
        let mut iterator = items.into_iter();
        let front = iterator.next();
        if front.is_none() {
            return vec![];
        }
        let mut bundler = Bundler::new(front.expect("Front has already been deemed to be some"));
        let mut bundled = vec![];
        for item in iterator {
            if item.pseudo_hash == bundler.value.pseudo_hash {
                bundler.pages.extend(item.pages.into_vec());
            } else {
                bundled.push(bundler.into());
                bundler = Bundler::new(item);
            }
        }
        bundled.push(bundler.into());
        bundled
    }
    pub fn build_table(
        items: impl IntoIterator<Item = PageInfoRow>,
        unit: Option<Unit>,
    ) -> tabled::Table {
        let sure_unit = unit.unwrap_or_default();
        let mut builder = Builder::default();

        builder.push_record(["Page", "Size", "Friendly name"]);
        let mut has_friendlies = false;
        for item in items {
            let standard: String = item.standard_name();
            has_friendlies |= !standard.is_empty();
            let size: String = match item.standard {
                Ok(p) => p.to_custom_size().as_unit_string(sure_unit),
                Err(e) => e.to_string(),
            };
            builder.push_record([item.pages.to_string(), size, standard]);
        }
        if !has_friendlies {
            builder.remove_column(builder.count_columns() - 1);
        }
        builder.build()
    }

    /// If [`standard`] is not a custom page, or error, outputs standard name.
    /// Otherwise outputs an empty string.
    pub fn standard_name(&self) -> String {
        match self.standard {
            Err(_) => "".into(),
            Ok(page) => match page {
                Page::American(us_paper) => us_paper.to_string(),
                Page::Standard(iso_paper) => {
                    let s = iso_paper.to_string();
                    if let Some(unprefixed) = s.strip_prefix('^') {
                        unprefixed.to_owned() + " (horizontal)"
                    } else {
                        s
                    }
                }
                _ => "".into(),
            },
        }
    }
    fn get_pseudo(pr: &anyhow::Result<Page>) -> String {
        match pr {
            Ok(o) => o.to_string(),
            Err(e) => e.to_string(),
        }
    }
    pub fn new(
        page: impl Into<DisplayableRange>,
        page_size: impl Into<anyhow::Result<Page>>,
    ) -> Self {
        let standard: anyhow::Result<Page> = page_size.into();
        PageInfoRow {
            pages: page.into(),
            pseudo_hash: PageInfoRow::get_pseudo(&standard),
            standard,
        }
    }
}
