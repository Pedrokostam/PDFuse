use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Neg, Sub},
};

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::errors::LengthParseError;

use super::{
    iso_paper::IsoPaper, page_size::PageSize, size::Size, unit::Unit, length::Length,
};

#[derive(Debug, PartialEq, Clone, Copy,Serialize,Deserialize)]
#[serde(try_from ="String")]
#[serde(into ="String")]
pub struct CustomSize {
    pub horizontal: Length,
    pub vertical: Length,
}

impl<T> Div<T> for CustomSize
where
    T: Copy + Into<f64>,
{
    type Output = CustomSize;

    fn div(self, rhs: T) -> Self::Output {
        CustomSize {
            horizontal: self.horizontal / rhs.into(),
            vertical: self.vertical / rhs.into(),
        }
    }
}

impl Default for CustomSize{
    fn default() -> Self {
        Self { horizontal: Length::zero(), vertical: Length::zero() }
    }
}

impl TryFrom<&str> for CustomSize {
    type Error = LengthParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_string(value)
    }
}

impl TryFrom<String> for CustomSize {
    type Error = LengthParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from_string(&value)
    }
}

impl From<CustomSize> for String{
    fn from(value: CustomSize) -> Self {
        value.to_string()
    }
}

impl<T> Mul<T> for CustomSize
where
    T: Copy + Into<f64>,
{
    type Output = CustomSize;

    fn mul(self, rhs: T) -> Self::Output {
        CustomSize {
            horizontal: self.horizontal * rhs.into(),
            vertical: self.vertical * rhs.into(),
        }
    }
}

impl Display for CustomSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match f.precision() {
            None => format!("{x} x {y}", x = self.horizontal(), y = self.vertical()),
            Some(prec) => format!(
                "{x:.prec$} x {y:.prec$}",
                x = self.horizontal(),
                y = self.vertical(),
                prec = prec
            ),
        };
        f.pad(&msg)
    }
}

impl Add<Self> for CustomSize {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        CustomSize {
            horizontal: self.horizontal + rhs.horizontal,
            vertical: self.vertical + rhs.vertical,
        }
    }
}

impl Neg for CustomSize {
    type Output = Self;

    fn neg(self) -> Self::Output {
        CustomSize {
            horizontal: -self.horizontal,
            vertical: -self.vertical,
        }
    }
}

impl Sub<Self> for CustomSize {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

impl Size for CustomSize {
    fn transposed(&self) -> Self {
        CustomSize {
            horizontal: self.vertical,
            vertical: self.horizontal,
        }
    }

    fn transpose(&mut self) {
        std::mem::swap(&mut self.vertical, &mut self.horizontal);
    }

    fn to_custom_size(&self) -> CustomSize {
        *self
    }

    fn horizontal(&self) -> Length {
        self.horizontal
    }

    fn vertical(&self) -> Length {
        self.vertical
    }

    fn fit_size(&self, other_size: &CustomSize) -> f64 {
        let x = self.horizontal() / other_size.horizontal();
        let y = self.vertical() / other_size.vertical();
        x.min(y)
    }
}

impl CustomSize {
    pub fn zero() -> Self {
        CustomSize {
            horizontal: Length::zero(),
            vertical: Length::zero(),
        }
    }
    /// Create new `CustomSize` where both dimension are in millimeters.
    pub fn from_millimeters<T>(horizontal: T, vertical: T) -> CustomSize
    where
        T: Into<f64>,
    {
        CustomSize {
            horizontal: Length::from_millimeters(horizontal),
            vertical: Length::from_millimeters(vertical),
        }
    }
    /// Create new `CustomSize` where both dimension are in centimeters.
    pub fn from_centimeters<T>(horizontal: T, vertical: T) -> CustomSize
    where
        T: Into<f64>,
    {
        CustomSize {
            horizontal: Length::from_centimeters(horizontal),
            vertical: Length::from_centimeters(vertical),
        }
    }
    /// Create new `CustomSize` where both dimension are in meters.
    pub fn from_meters<T>(horizontal: T, vertical: T) -> CustomSize
    where
        T: Into<f64>,
    {
        CustomSize {
            horizontal: Length::from_meters(horizontal),
            vertical: Length::from_meters(vertical),
        }
    }
    /// Create new `CustomSize` where both dimension are in inches.
    pub fn from_inches<T>(horizontal: T, vertical: T) -> CustomSize
    where
        T: Into<f64>,
    {
        CustomSize {
            horizontal: Length::from_inches(horizontal),
            vertical: Length::from_inches(vertical),
        }
    }
    /// Create new `CustomSize` where both dimension are in points.
    pub fn from_points<T>(horizontal: T, vertical: T) -> CustomSize
    where
        T: Into<f64>,
    {
        CustomSize {
            horizontal: Length::from_points(horizontal),
            vertical: Length::from_points(vertical),
        }
    }
    pub fn try_from_string(text: &str) -> Result<Self, LengthParseError> {
        static LAST_UNIT_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(?i)([A-Z]+)[\s;,]*$").unwrap());
        let last_unit_str = LAST_UNIT_REGEX
            .captures(text)
            .and_then(|c| c.get(0))
            .map(|c| c.as_str());
        let last_unit = match last_unit_str {
            None => None,
            Some(x) => Some(Unit::from_string(x)?),
        };

        let unit_ditance_1_res_opt = Length::from_string_with_default_result(text, last_unit);
        let unit_distance_1_res = unit_ditance_1_res_opt?;
        let unit_distance_1 = unit_distance_1_res.parsed_value;
        let start = unit_distance_1_res.end_position;
        let default_unit = unit_distance_1_res.unit;
        let unit_distance_2_opt =
            Length::from_string_with_default(&text[start..], Some(default_unit));
        let unit_distance_2 = match unit_distance_2_opt {
            Ok(length) => length.parsed_value,
            Err(_) => unit_distance_1,
        };
        Ok(CustomSize {
            horizontal: unit_distance_1,
            vertical: unit_distance_2,
        })
    }
}

impl From<IsoPaper> for CustomSize {
    fn from(value: IsoPaper) -> Self {
        value.to_custom_size()
    }
}

impl From<PageSize> for CustomSize {
    fn from(value: PageSize) -> Self {
        match value {
            PageSize::Standard(iso_paper) => iso_paper.into(),
            PageSize::Custom(custom_size) => custom_size,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_fun_call)]

    use super::*;
    #[test]
    fn parsing_custom() {
        let test_vals = vec![
            (
                "12.3437m-22.3437m",
                CustomSize::from_meters(12.343737, 22.343737),
            ),
            (
                " 12.3437m x 22.3437m ",
                CustomSize::from_meters(12.3437, 22.3437),
            ),
            (
                " 12.3437m x 22.3437m ",
                CustomSize::from_meters(12.3437, 22.3437),
            ),
            (" 12.3437m ", CustomSize::from_meters(12.3437, 12.3437)),
            (
                " 12.3437 x 22.3437m ",
                CustomSize::from_meters(12.3437, 22.3437),
            ),
            (
                " 12.3437m x 22.3437 ",
                CustomSize::from_meters(12.3437, 22.3437),
            ),
            (
                " 12.3437  22.3437m ",
                CustomSize::from_meters(12.3437, 22.3437),
            ),
            (
                " 12.3437 mm 22.3437pt ",
                CustomSize {
                    horizontal: Length::from_millimeters(12.3437),
                    vertical: Length::from_points(22.3437),
                },
            ),
            (
                " 12.3437 mm x 22.3437pt ",
                CustomSize {
                    horizontal: Length::from_millimeters(12.3437),
                    vertical: Length::from_points(22.3437),
                },
            ),
        ];
        for (text, paper) in test_vals {
            let parsed =
                CustomSize::try_from_string(text).expect(&format!("Failed parsing '{}'", text));
            assert_eq!(parsed, paper, "{}", text);
        }
    }
}
