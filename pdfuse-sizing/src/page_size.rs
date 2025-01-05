use crate::custom_size::CustomSize;
use crate::iso_paper::IsoPaper;
use crate::length::Length;
use crate::size::Size;
use crate::errors::{IsoPaperError,PageSizeError};
use std::convert::From;
use std::fmt::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy,Deserialize,Serialize)]
#[serde(try_from ="String")]
#[serde(into ="String")]
pub enum PageSize {
    Standard(IsoPaper),
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

    fn try_from(value:String) -> Result<Self, Self::Error> {
        Self::try_from_string(&value)
    }
}
impl From<PageSize> for String{
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
impl PageSize {
    pub fn try_from_string(text: &str) -> Result<Self, PageSizeError> {
        match IsoPaper::try_from_string(text) {
            Ok(iso) => Ok(iso.into()),
            Err(e) if e == IsoPaperError::NotIsoPage => Err(e.into()),
            Err(_) => Ok(CustomSize::try_from_string(text)?.into()),
        }
    }
}
impl Display for PageSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PageSize::Standard(iso_paper) => iso_paper.fmt(f),
            PageSize::Custom(custom_size) => custom_size.fmt(f),
        }
    }
}
impl Size for PageSize {
    fn transposed(&self) -> Self {
        match self {
            PageSize::Standard(iso_paper) => PageSize::Standard(iso_paper.transposed()),
            PageSize::Custom(custom_size) => PageSize::Custom(custom_size.transposed()),
        }
    }

    fn transpose(&mut self) {
        match self {
            PageSize::Standard(iso_paper) => iso_paper.transpose(),
            PageSize::Custom(custom_size) => custom_size.transpose(),
        }
    }

    fn to_custom_size(&self) -> CustomSize {
        match self {
            PageSize::Standard(iso_paper) => iso_paper.to_custom_size(),
            PageSize::Custom(custom_size) => custom_size.to_custom_size(),
        }
    }

    fn horizontal(&self) -> Length {
        match self {
            PageSize::Standard(iso_paper) => iso_paper.horizontal(),
            PageSize::Custom(custom_size) => custom_size.horizontal(),
        }
    }

    fn vertical(&self) -> Length {
        match self {
            PageSize::Standard(iso_paper) => iso_paper.vertical(),
            PageSize::Custom(custom_size) => custom_size.vertical(),
        }
    }

    fn fit_size(&self, other_size: &CustomSize) -> f64 {
        self.to_custom_size().fit_size(other_size)
    }
}
impl Default for PageSize {
    fn default() -> Self {
        PageSize::Standard(IsoPaper::default())
    }
}
