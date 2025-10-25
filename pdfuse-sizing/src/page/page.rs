use crate::error::{IsoPaperError, PageSizeError};
use crate::page::{CustomPage, IsoPaper, UsPaper};
use crate::{Length, Size, TransposableSize};
use serde::{Deserialize, Serialize};
use std::convert::From;
use std::fmt::Display;

#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Serialize)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub enum Page {
    Standard(IsoPaper),
    American(UsPaper),
    Custom(CustomPage),
}
impl TryFrom<&str> for Page {
    type Error = PageSizeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_string(value)
    }
}
impl TryFrom<String> for Page {
    type Error = PageSizeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from_string(&value)
    }
}
impl From<Page> for String {
    fn from(value: Page) -> Self {
        value.to_string()
    }
}
impl From<IsoPaper> for Page {
    fn from(value: IsoPaper) -> Self {
        Page::Standard(value)
    }
}
impl From<CustomPage> for Page {
    fn from(value: CustomPage) -> Self {
        Page::Custom(value)
    }
}
impl From<UsPaper> for Page {
    fn from(value: UsPaper) -> Self {
        Page::American(value)
    }
}
impl Page {
    pub fn try_from_string(text: &str) -> Result<Self, PageSizeError> {
        let trimmed = text.trim();
        match IsoPaper::try_from_string(trimmed) {
            Ok(iso) => Ok(iso.into()),
            // NotIsoPage is the first error we can have when parsing ISO
            // It tells that [ABC]\d+ sequence was not found
            Err(e) if e != IsoPaperError::NotIsoPage => Err(e.into()),
            Err(_) => {
                let has_numbers = trimmed.chars().any(|c| c.is_numeric());
                match has_numbers {
                    false => match UsPaper::try_from_string(trimmed) {
                        Ok(u) => Ok(u.into()),
                        Err(ue) => Err(ue.into()),
                    },
                    true => match CustomPage::try_from_string(trimmed) {
                        Ok(c) => Ok(c.into()),
                        Err(ce) => Err(ce.into()),
                    },
                }
            }
        }
    }
}
impl Display for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Page::Standard(iso_paper) => iso_paper.fmt(f),
            Page::Custom(custom_size) => custom_size.fmt(f),
            Page::American(us_paper) => us_paper.fmt(f),
        }
    }
}
impl TransposableSize for Page {
    fn transposed(&self) -> Self {
        match self {
            Page::Standard(iso_paper) => Page::Standard(iso_paper.transposed()),
            Page::Custom(custom_size) => Page::Custom(custom_size.transposed()),
            Page::American(us_paper) => Page::Custom(us_paper.to_custom_size()),
        }
    }

    fn transpose(&mut self) {
        match self {
            Page::Standard(iso_paper) => iso_paper.transpose(),
            Page::Custom(custom_size) => custom_size.transpose(),
            Page::American(us_paper) => us_paper.to_custom_size().transpose(),
        }
    }
}
impl Size for Page {
    fn to_custom_size(&self) -> CustomPage {
        match self {
            Page::Standard(iso_paper) => iso_paper.to_custom_size(),
            Page::Custom(custom_size) => custom_size.to_custom_size(),
            Page::American(us_paper) => us_paper.to_custom_size(),
        }
    }

    fn horizontal(&self) -> Length {
        match self {
            Page::Standard(iso_paper) => iso_paper.horizontal(),
            Page::Custom(custom_size) => custom_size.horizontal(),
            Page::American(us_paper) => us_paper.horizontal(),
        }
    }

    fn vertical(&self) -> Length {
        match self {
            Page::Standard(iso_paper) => iso_paper.vertical(),
            Page::Custom(custom_size) => custom_size.vertical(),
            Page::American(us_paper) => us_paper.vertical(),
        }
    }

    fn fit_size(&self, other_size: &CustomPage) -> f64 {
        self.to_custom_size().fit_size(other_size)
    }
}
impl Default for Page {
    /// Standard ISO paper size - A4.
    fn default() -> Self {
        Page::Standard(IsoPaper::default())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_fun_call)]

    use super::*;
    #[test]
    fn parsing_custom() {
        let test_vals: Vec<(&'static str, Page)> = vec![
            (
                "12.3437m-22.3437m",
                CustomPage::from_meters(12.3437, 22.3437).into(),
            ),
            (
                " 12.3437m x 22.3437m ",
                CustomPage::from_meters(12.3437, 22.3437).into(),
            ),
            (
                " 12.3437m x 22.3437m ",
                CustomPage::from_meters(12.3437, 22.3437).into(),
            ),
            (
                " 12.3437m ",
                CustomPage::from_meters(12.3437, 12.3437).into(),
            ),
            (
                " 12.3437 x 22.3437m ",
                CustomPage::from_meters(12.3437, 22.3437).into(),
            ),
            (
                " 12.3437m x 22.3437 ",
                CustomPage::from_meters(12.3437, 22.3437).into(),
            ),
            (
                " 12.3437  22.3437m ",
                CustomPage::from_meters(12.3437, 22.3437).into(),
            ),
            (
                " 12.3437 mm 22.3437pt ",
                CustomPage {
                    horizontal: Length::from_millimeters(12.3437),
                    vertical: Length::from_points(22.3437),
                }
                .into(),
            ),
            (
                " 12.3437 mm x 22.3437pt ",
                CustomPage {
                    horizontal: Length::from_millimeters(12.3437),
                    vertical: Length::from_points(22.3437),
                }
                .into(),
            ),
            ("12cm", CustomPage::from_centimeters(12, 12).into()),
        ];
        for (text, paper) in test_vals {
            let parsed =
                Page::try_from_string(text).expect(&format!("Failed parsing '{text}'"));
            assert_eq!(parsed, paper, "{text}");
        }
    }
}
