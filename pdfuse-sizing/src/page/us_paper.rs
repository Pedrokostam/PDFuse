use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::CustomSize;
use crate::error::UsPaperError;
use crate::{Length, Size};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub enum UsPaper {
    Letter,
    Legal,
    Tabloid,
    Ledger,
    Executive,
}

impl TryFrom<String> for UsPaper {
    type Error = UsPaperError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from_string(&value)
    }
}

impl From<UsPaper> for String {
    fn from(value: UsPaper) -> Self {
        value.to_string()
    }
}

impl Display for UsPaper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            UsPaper::Letter => "Letter",
            UsPaper::Ledger => "Ledger",
            UsPaper::Tabloid => "Tabloid",
            UsPaper::Legal => "Legal",
            UsPaper::Executive => "Executive",
        };
        f.pad(txt)
    }
}

impl UsPaper {
    pub fn try_from_string(text: &str) -> Result<Self, UsPaperError> {
        let upper = text.trim().to_ascii_uppercase();
        match upper.as_str() {
            "LETTER" => Ok(UsPaper::Letter),
            "LEGAL" => Ok(UsPaper::Legal),
            "TABLOID" => Ok(UsPaper::Tabloid),
            "LEDGER" => Ok(UsPaper::Ledger),
            "EXECUTIVE" => Ok(UsPaper::Executive),
            _ => Err(UsPaperError {
                invalid: text.to_owned(),
            }),
        }
    }
}

impl Size for UsPaper {
    fn to_custom_size(&self) -> CustomSize {
        match self {
            UsPaper::Letter => CustomSize::from_inches(8.5, 11.0),
            UsPaper::Ledger => CustomSize::from_inches(17.0, 11.0),
            UsPaper::Tabloid => CustomSize::from_inches(11.0, 17.0),
            UsPaper::Legal => CustomSize::from_inches(8.5, 14.0),
            UsPaper::Executive => CustomSize::from_inches(7.25, 10.5),
        }
    }

    fn horizontal(&self) -> Length {
        self.to_custom_size().horizontal
    }

    fn vertical(&self) -> Length {
        self.to_custom_size().vertical
    }

    fn fit_size(&self, other_size: &CustomSize) -> f64 {
        self.to_custom_size().fit_size(other_size)
    }
}
