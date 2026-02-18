use std::str::FromStr;

use derive_more::Display;
use serde::{Deserialize, Serialize};

use super::CustomPage;
use crate::error::UsPaperError;
use crate::{Length, Size};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize, Display)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub enum UsPaper {
    Letter,
    Legal,
    Tabloid,
    Ledger,
    Executive,
}

impl FromStr for UsPaper {
    type Err = UsPaperError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from_string(s)
    }
}

impl TryFrom<&str> for UsPaper {
    type Error = UsPaperError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_string(value)
    }
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
    fn to_custom_size(&self) -> CustomPage {
        match self {
            UsPaper::Letter => CustomPage::from_inches(8.5, 11.0),
            UsPaper::Ledger => CustomPage::from_inches(17.0, 11.0),
            UsPaper::Tabloid => CustomPage::from_inches(11.0, 17.0),
            UsPaper::Legal => CustomPage::from_inches(8.5, 14.0),
            UsPaper::Executive => CustomPage::from_inches(7.25, 10.5),
        }
    }

    fn horizontal(&self) -> Length {
        self.to_custom_size().horizontal
    }

    fn vertical(&self) -> Length {
        self.to_custom_size().vertical
    }

    fn fit_size(&self, other_size: &CustomPage) -> f64 {
        self.to_custom_size().fit_size(other_size)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn get_custom(in_x: f64, in_y: f64) -> CustomPage {
        CustomPage {
            horizontal: Length::from_inches(in_x),
            vertical: Length::from_inches(in_y),
        }
    }
    #[test]
    fn parsing() {
        let txts = vec![
            "  lEtTeR ",
            " lEgAl ",
            "  TaBlOiD  ",
            " lEdGeR  ",
            " eXeCuTiVe ",
        ];
        for t in txts {
            assert!(UsPaper::try_from_string(t).is_ok(), "{t}");
            assert!(
                UsPaper::try_from_string(&t.to_ascii_uppercase()).is_ok(),
                "{t} uppercase"
            );
            assert!(
                UsPaper::try_from_string(&t.to_ascii_lowercase()).is_ok(),
                "{t} lowercase"
            );
        }
    }

    #[test]
    fn us_size() {
        let vals = vec![
            (UsPaper::Letter, get_custom(8.5, 11.0)),
            (UsPaper::Ledger, get_custom(17.0, 11.0)),
            (UsPaper::Tabloid, get_custom(11.0, 17.0)),
            (UsPaper::Executive, get_custom(7.25, 10.5)),
            (UsPaper::Legal, get_custom(8.5, 14.0)),
        ];
        for (manual, custom) in vals {
            assert_eq!(manual.to_custom_size(), custom, "{manual} to {custom}");
        }
    }
}
