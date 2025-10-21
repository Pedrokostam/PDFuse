use crate::error::{PageSizeError,IsoPaperError};
use crate::page::{CustomSize,IsoPaper,UsPaper};
use crate::{Length, Size, TransposableSize };
use serde::{Deserialize, Serialize};
use std::convert::From;
use std::fmt::Display;

#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Serialize)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub enum PageSize {
    Standard(IsoPaper),
    American(UsPaper),
    Custom(CustomSize),
}
impl TryFrom<&str> for PageSize {
    type Error = PageSizeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_string(value)
    }
}
impl TryFrom<String> for PageSize {
    type Error = PageSizeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from_string(&value)
    }
}
impl From<PageSize> for String {
    fn from(value: PageSize) -> Self {
        value.to_string()
    }
}
impl From<IsoPaper> for PageSize {
    fn from(value: IsoPaper) -> Self {
        PageSize::Standard(value)
    }
}
impl From<CustomSize> for PageSize {
    fn from(value: CustomSize) -> Self {
        PageSize::Custom(value)
    }
}
impl From<UsPaper> for PageSize {
    fn from(value: UsPaper) -> Self {
        PageSize::American(value)
    }
}
impl PageSize {
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
                    true => match CustomSize::try_from_string(trimmed) {
                        Ok(c) => Ok(c.into()),
                        Err(ce) => Err(ce.into()),
                    },
                }
            }
        }
    }
}
impl Display for PageSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PageSize::Standard(iso_paper) => iso_paper.fmt(f),
            PageSize::Custom(custom_size) => custom_size.fmt(f),
            PageSize::American(us_paper) => us_paper.fmt(f),
        }
    }
}
impl TransposableSize for PageSize {
    fn transposed(&self) -> Self {
        match self {
            PageSize::Standard(iso_paper) => PageSize::Standard(iso_paper.transposed()),
            PageSize::Custom(custom_size) => PageSize::Custom(custom_size.transposed()),
            PageSize::American(us_paper) => PageSize::Custom(us_paper.to_custom_size()),
        }
    }

    fn transpose(&mut self) {
        match self {
            PageSize::Standard(iso_paper) => iso_paper.transpose(),
            PageSize::Custom(custom_size) => custom_size.transpose(),
            PageSize::American(us_paper) => us_paper.to_custom_size().transpose(),
        }
    }
}
impl Size for PageSize {
    fn to_custom_size(&self) -> CustomSize {
        match self {
            PageSize::Standard(iso_paper) => iso_paper.to_custom_size(),
            PageSize::Custom(custom_size) => custom_size.to_custom_size(),
            PageSize::American(us_paper) => us_paper.to_custom_size(),
        }
    }

    fn horizontal(&self) -> Length {
        match self {
            PageSize::Standard(iso_paper) => iso_paper.horizontal(),
            PageSize::Custom(custom_size) => custom_size.horizontal(),
            PageSize::American(us_paper) => us_paper.horizontal(),
        }
    }

    fn vertical(&self) -> Length {
        match self {
            PageSize::Standard(iso_paper) => iso_paper.vertical(),
            PageSize::Custom(custom_size) => custom_size.vertical(),
            PageSize::American(us_paper) => us_paper.vertical(),
        }
    }

    fn fit_size(&self, other_size: &CustomSize) -> f64 {
        self.to_custom_size().fit_size(other_size)
    }
}
impl Default for PageSize {
    /// Standard ISO paper size - A4.
    fn default() -> Self {
        PageSize::Standard(IsoPaper::default())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_fun_call)]

    use super::*;
    #[test]
    fn parsing_custom() {
        let test_vals: Vec<(&'static str, PageSize)> = vec![
            (
                "12.3437m-22.3437m",
                CustomSize::from_meters(12.343737, 22.343737).into(),
            ),
            (
                " 12.3437m x 22.3437m ",
                CustomSize::from_meters(12.3437, 22.3437).into(),
            ),
            (
                " 12.3437m x 22.3437m ",
                CustomSize::from_meters(12.3437, 22.3437).into(),
            ),
            (
                " 12.3437m ",
                CustomSize::from_meters(12.3437, 12.3437).into(),
            ),
            (
                " 12.3437 x 22.3437m ",
                CustomSize::from_meters(12.3437, 22.3437).into(),
            ),
            (
                " 12.3437m x 22.3437 ",
                CustomSize::from_meters(12.3437, 22.3437).into(),
            ),
            (
                " 12.3437  22.3437m ",
                CustomSize::from_meters(12.3437, 22.3437).into(),
            ),
            (
                " 12.3437 mm 22.3437pt ",
                CustomSize {
                    horizontal: Length::from_millimeters(12.3437),
                    vertical: Length::from_points(22.3437),
                }
                .into(),
            ),
            (
                " 12.3437 mm x 22.3437pt ",
                CustomSize {
                    horizontal: Length::from_millimeters(12.3437),
                    vertical: Length::from_points(22.3437),
                }
                .into(),
            ),
            ("12cm", CustomSize::from_centimeters(12, 12).into()),
        ];
        for (text, paper) in test_vals {
            let parsed =
                PageSize::try_from_string(text).expect(&format!("Failed parsing '{text}'"));
            assert_eq!(parsed, paper, "{text}");
        }
    }
}

