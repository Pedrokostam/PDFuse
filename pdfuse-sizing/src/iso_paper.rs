use core::f64;
use std::fmt::{Debug, Display};

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::errors::IsoPaperError;

use super::{CustomSize, Length, Size};

/// All lengths of paper from A0 to A13, first element vertical, last horizontal
pub(crate) const A_LENGTHS: &[f64] = &[
    1189.0, 841.0, 594.0, 420.0, 297.0, 210.0, 148.0, 105.0, 74.0, 52.0, 37.0, 26.0, 18.0, 13.0,
    9.0,
];
/// All lengths of paper from B0 to B13, first element vertical, last horizontal
pub(crate) const B_LENGTHS: &[f64] = &[
    1414.0, 1000.0, 707.0, 500.0, 353.0, 250.0, 176.0, 125.0, 88.0, 62.0, 44.0, 31.0, 22.0, 15.0,
    11.0,
];

/// All lengths of paper from C0 to C13, first element vertical, last horizontal
pub(crate) const C_LENGTHS: &[f64] = &[
    1297.0, 917.0, 648.0, 458.0, 324.0, 229.0, 162.0, 114.0, 81.0, 57.0, 40.0, 28.0, 20.0, 14.0,
    10.0,
];

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IsoPaperType {
    A,
    B,
    C,
}
impl Display for IsoPaperType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let letter = match self {
            IsoPaperType::A => "A",
            IsoPaperType::B => "B",
            IsoPaperType::C => "C",
        };
        f.pad(letter)
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct IsoPaper {
    paper_type: IsoPaperType,
    paper_size: i8,
    is_transposed: bool,
    short: Length,
    long: Length,
}
impl TryFrom<String> for IsoPaper {
    type Error = IsoPaperError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from_string(&value)
    }
}
impl From<IsoPaper> for String {
    fn from(value: IsoPaper) -> Self {
        value.to_string()
    }
}

impl IsoPaper {
    pub fn paper_type(&self) -> &IsoPaperType {
        &self.paper_type
    }
    pub fn paper_size(&self) -> &i8 {
        &self.paper_size
    }
    pub fn is_transposed(&self) -> &bool {
        &self.is_transposed
    }
    pub fn short(&self) -> &Length {
        &self.short
    }
    pub fn long(&self) -> &Length {
        &self.long
    }

    pub fn iso_name(&self) -> String {
        format!(
            "{transposed}{typ}{size}",
            typ = self.paper_type,
            size = self.paper_size,
            transposed = if self.is_transposed { "^" } else { "" }
        )
    }

    pub fn a(size: i8) -> IsoPaper {
        Self::new(IsoPaperType::A, size, false)
    }
    pub fn a_transposed(size: i8) -> IsoPaper {
        Self::new(IsoPaperType::A, size, true)
    }
    pub fn b(size: i8) -> IsoPaper {
        Self::new(IsoPaperType::B, size, false)
    }
    pub fn b_transposed(size: i8) -> IsoPaper {
        Self::new(IsoPaperType::B, size, true)
    }
    pub fn c(size: i8) -> IsoPaper {
        Self::new(IsoPaperType::C, size, false)
    }
    pub fn c_transposed(size: i8) -> IsoPaper {
        Self::new(IsoPaperType::C, size, true)
    }

    pub fn new(paper_type: IsoPaperType, paper_size: i8, is_transposed: bool) -> Self {
        assert!(
            paper_size >= 0 && (paper_size as usize) < A_LENGTHS.len() - 1,
            "For ISO 216 paper only sizes up to {} are supported (input was {})",
            A_LENGTHS.len() - 1,
            paper_size
        );
        let vertical = match paper_type {
            IsoPaperType::A => A_LENGTHS[paper_size as usize],
            IsoPaperType::B => B_LENGTHS[paper_size as usize],
            IsoPaperType::C => C_LENGTHS[paper_size as usize],
        };
        let horizontal = match paper_type {
            IsoPaperType::A => A_LENGTHS[(paper_size + 1) as usize],
            IsoPaperType::B => B_LENGTHS[(paper_size + 1) as usize],
            IsoPaperType::C => C_LENGTHS[(paper_size + 1) as usize],
        };
        IsoPaper {
            is_transposed,
            paper_size,
            paper_type,
            short: Length::from_millimeters(horizontal),
            long: Length::from_millimeters(vertical),
        }
    }

    pub fn try_from_string(text: &str) -> Result<Self, IsoPaperError> {
        static PAPER_SIZE_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(?i)((?<Transposed>\^\s*)?(?<Paper>[A-Z])\s*(?<Size>-?\d+))").unwrap()
        });
        let captures = PAPER_SIZE_REGEX
            .captures(text)
            .ok_or(IsoPaperError::NotIsoPage)?;
        let size_str = captures
            .name("Size")
            .ok_or(IsoPaperError::NoSizeSpecified)?;

        let paper_size = size_str.as_str().parse::<i64>()?;
        if paper_size < 0 || (paper_size as usize) > A_LENGTHS.len() - 1 {
            return Err(IsoPaperError::InvalidSize(paper_size));
        }

        let paper_str = captures
            .name("Paper")
            .ok_or(IsoPaperError::NoTypeSpecied)?
            .as_str();

        let paper_type = match paper_str {
            "A" | "a" => Ok(IsoPaperType::A),
            "B" | "b" => Ok(IsoPaperType::B),
            "C" | "c" => Ok(IsoPaperType::C),
            _ => Err(IsoPaperError::InvalidType(paper_str.to_owned())),
        }?;
        let is_transposed = captures.name("Transposed").is_some();
        Ok(IsoPaper::new(paper_type, paper_size as i8, is_transposed))
    }
}
impl Display for IsoPaper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(&self.iso_name())
    }
}
impl Size for IsoPaper {
    fn transposed(&self) -> Self {
        IsoPaper {
            is_transposed: !self.is_transposed,
            ..*self
        }
    }

    fn transpose(&mut self) {
        self.is_transposed = !self.is_transposed;
    }

    fn horizontal(&self) -> Length {
        if !self.is_transposed {
            self.short
        } else {
            self.long
        }
    }

    fn vertical(&self) -> Length {
        if !self.is_transposed {
            self.long
        } else {
            self.short
        }
    }

    fn to_custom_size(&self) -> CustomSize {
        CustomSize {
            horizontal: self.horizontal(),
            vertical: self.vertical(),
        }
    }

    fn fit_size(&self, other_size: &CustomSize) -> f64 {
        self.to_custom_size().fit_size(other_size)
    }
}
impl Default for IsoPaper {
    fn default() -> Self {
        Self::new(IsoPaperType::A, 4, false)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_fun_call)]
    use super::*;
    fn get_custom(mm_x: i64, mm_y: i64) -> CustomSize {
        CustomSize {
            horizontal: Length::from_millimeters(mm_x as f64),
            vertical: Length::from_millimeters(mm_y as f64),
        }
    }
    #[test]
    fn same_number_of_standard_sizes() {
        assert_eq!(
            A_LENGTHS.len(),
            B_LENGTHS.len(),
            "A and B sizes have different number of elements"
        );
        assert_eq!(
            A_LENGTHS.len(),
            C_LENGTHS.len(),
            "A and C sizes have different number of elements"
        );
    }
    #[test]
    fn parsing_iso() {
        let test_vals = vec![
            ("a4", IsoPaper::a(4)),
            ("B4", IsoPaper::b(4)),
            ("c4", IsoPaper::c(4)),
            ("   C   12   ", IsoPaper::c(12)),
            ("^a4", IsoPaper::a_transposed(4)),
        ];
        for (text, paper) in test_vals {
            let parsed =
                IsoPaper::try_from_string(text).expect(&format!("Failed parsing '{text}'" ));
            assert_eq!(parsed, paper);
        }
    }
    #[test]
    fn iso_size() {
        let vals = vec![
            (IsoPaper::a(0), get_custom(841, 1189)),
            (IsoPaper::a(4), get_custom(210, 297)),
            (IsoPaper::a(6), get_custom(105, 148)),
            (IsoPaper::b(0), get_custom(1000, 1414)),
            (IsoPaper::b(4), get_custom(250, 353)),
            (IsoPaper::b(6), get_custom(125, 176)),
            (IsoPaper::c(0), get_custom(917, 1297)),
            (IsoPaper::c(4), get_custom(229, 324)),
            (IsoPaper::c(6), get_custom(114, 162)),
        ];
        for (manual, custom) in vals {
            assert_eq!(manual.to_custom_size(), custom, "{manual} to {custom}");
        }
    }
    #[test]
    fn parse_error_tests() {
        let texts = vec![
            ("a-1", IsoPaperError::InvalidSize(-1)),
            ("z4", IsoPaperError::InvalidType("z".to_owned())),
            ("21.37cm", IsoPaperError::NotIsoPage),
        ];
        for (text, expected_error) in texts {
            let parsed_iso_error = IsoPaper::try_from_string(text).expect_err("It should be wrong");
            assert_eq!(
                parsed_iso_error, expected_error,
                "{text} to {expected_error}"
            );
        }
    }
}
