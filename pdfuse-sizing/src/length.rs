use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign},
};

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::errors::{LengthParseError, UnitParseError};

use super::{parsing::ParseResult, unit::Unit};

#[derive(Debug, PartialEq, Clone, Copy, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct Length {
    pub(crate) base_value: f64,
}
impl From<Length> for String {
    fn from(value: Length) -> Self {
        value.as_unit_str(Length::BASE_UNIT)
    }
}
impl TryFrom<&str> for Length {
    type Error = LengthParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_string(value)
    }
}
impl TryFrom<String> for Length {
    type Error = LengthParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from_string(&value)
    }
}
impl Length {
    pub const BASE_UNIT: Unit = Unit::Millimeter;
    pub fn zero() -> Self {
        Length { base_value: 0.0 }
    }
    fn change_to_base(value: f64, from: Unit) -> f64 {
        Unit::change_unit(value, from, Self::BASE_UNIT)
    }
    pub fn from_unit(value: impl Into<f64>, unit: Unit) -> Self {
        Length {
            base_value: Self::change_to_base(value.into(), unit),
        }
    }
    pub fn as_unit_str(&self, unit: Unit) -> String {
        format!("{} {}", self.as_unit(unit), unit.unit_symbol())
    }
    pub fn as_unit(&self, unit: Unit) -> f64 {
        Unit::change_unit(self.base_value, Unit::Millimeter, unit)
    }
    pub fn m(&self) -> f64 {
        self.as_unit(Unit::Meter)
    }
    pub fn from_meters(meters: impl Into<f64>) -> Self {
        Self::from_unit(meters, Unit::Meter)
    }
    pub fn mm(&self) -> f64 {
        self.as_unit(Unit::Millimeter)
    }
    pub fn from_millimeters(millimeters: impl Into<f64>) -> Self {
        Self::from_unit(millimeters, Unit::Millimeter)
    }
    pub fn from_centimeters(millimeters: impl Into<f64>) -> Self {
        Self::from_unit(millimeters, Unit::Centimeter)
    }
    pub fn inch(&self) -> f64 {
        self.as_unit(Unit::Inch)
    }
    pub fn from_inches(inches: impl Into<f64>) -> Self {
        Self::from_unit(inches, Unit::Inch)
    }
    pub fn pt(&self) -> f64 {
        self.as_unit(Unit::Point)
    }
    pub fn from_points(points: impl Into<f64>) -> Self {
        Self::from_unit(points, Unit::Point)
    }
}
impl Length {
    pub(crate) fn from_string_with_default_result(
        text: &str,
        default_unit: Option<Unit>,
    ) -> Result<ParseResult, LengthParseError> {
        static UNIT_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(?i)(?<Value>[\d\.]+)\s*(?<Unit>[A-Z]+)?").unwrap());
        let captures = UNIT_REGEX
            .captures(text)
            .ok_or(LengthParseError::NoValueSpecified)?;
        let value_capt = captures
            .name("Value")
            .ok_or(LengthParseError::NoValueSpecified)?;
        let Ok(value) = value_capt.as_str().parse::<f64>() else {
            return Err(LengthParseError::NoValueSpecified);
        };
        let end_position = captures.get(0).unwrap().range().end;

        let unit_capt = captures.name("Unit").map(|c| c.as_str());
        let (parsed_value, unit) = match (unit_capt, default_unit, value) {
            (Some(unit_str), Some(default_unit_m), _) => {
                let unit = Unit::from_string(unit_str).unwrap_or(default_unit_m);
                (Length::from_unit(value, unit), unit)
            }
            (Some(unit_str), None, _) => {
                let unit = Unit::from_string(unit_str)?;
                (Length::from_unit(value, unit), unit)
            }
            (None, Some(default_unit_m), _) => {
                (Length::from_unit(value, default_unit_m), default_unit_m)
            }
            // if the parsed value is zero, skip checking for unit
            // it does not matter
            (None, None, 0.0) => (
                Length::from_unit(value, Length::BASE_UNIT),
                Length::BASE_UNIT,
            ),
            _ => Err(UnitParseError::NoUnitSpecified)?,
        };
        Ok(ParseResult {
            parsed_value,
            unit,
            end_position,
        })
    }
    pub(crate) fn from_string_with_default(
        text: &str,
        default_unit: Option<Unit>,
    ) -> Result<ParseResult, LengthParseError> {
        Self::from_string_with_default_result(text, default_unit)
    }
    pub fn try_from_string(text: &str) -> Result<Length, LengthParseError> {
        Self::from_string_with_default(text, None).map(|pr| pr.parsed_value)
    }
}
impl Add<Self> for Length {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Length {
            base_value: self.base_value + rhs.base_value,
        }
    }
}
impl AddAssign<Self> for Length {
    fn add_assign(&mut self, rhs: Self) {
        self.base_value += rhs.base_value;
    }
}
impl Neg for Length {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Length {
            base_value: -self.base_value,
        }
    }
}

impl Sub<Self> for Length {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Length {
            base_value: self.base_value - rhs.base_value,
        }
    }
}
impl SubAssign<Self> for Length {
    fn sub_assign(&mut self, rhs: Self) {
        self.base_value -= rhs.base_value;
    }
}
impl<T> Div<T> for Length
where
    T: Copy + Into<f64>,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Length {
            base_value: self.base_value / rhs.into(),
        }
    }
}
impl Div<Self> for Length {
    type Output = f64;

    fn div(self, rhs: Self) -> Self::Output {
        self.base_value / rhs.base_value
    }
}
impl<T> Mul<T> for Length
where
    T: Copy + Into<f64>,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Length {
            base_value: self.base_value * rhs.into(),
        }
    }
}
impl Display for Length {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{value} {unit:.2}",
            value = self.mm(),
            unit = Unit::Millimeter.unit_symbol()
        )
    }
}
impl From<Length> for printpdf::units::Mm {
    fn from(val: Length) -> Self {
        printpdf::units::Mm(val.mm() as f32)
    }
}

impl From<Length> for printpdf::units::Pt {
    fn from(val: Length) -> Self {
        printpdf::units::Pt(val.pt() as f32)
    }
}


#[cfg(test)]
mod tests {
    #![allow(clippy::expect_fun_call)]
    use super::*;
    #[test]
    fn parsing_length() {
        let texts = vec![
            ("121mm", Length::from_millimeters(121.0)),
            ("21.37mm", Length::from_millimeters(21.37)),
            ("121 mm", Length::from_millimeters(121.0)),
            ("121 milli", Length::from_millimeters(121.0)),
            ("121 millimeters", Length::from_millimeters(121.0)),
            ("121cm", Length::from_centimeters(121.0)),
            ("21.37cm", Length::from_centimeters(21.37)),
            ("121 cm", Length::from_centimeters(121.0)),
            ("121 centimeters", Length::from_centimeters(121.0)),
            ("121m", Length::from_meters(121.0)),
            ("21.37m", Length::from_meters(21.37)),
            ("121 m", Length::from_meters(121.0)),
            ("121 meters", Length::from_meters(121.0)),
            ("121in", Length::from_inches(121.0)),
            ("21.37in", Length::from_inches(21.37)),
            ("121 in", Length::from_inches(121.0)),
            ("121 inches", Length::from_inches(121.0)),
            ("121pt", Length::from_points(121.0)),
            ("21.37pt", Length::from_points(21.37)),
            ("121 pt", Length::from_points(121.0)),
            ("121 points", Length::from_points(121.0)),
        ];
        for (text, length) in texts {
            let parsed =
                Length::try_from_string(text).expect(&format!("Could not parse {}", &text));
            assert_eq!(parsed, length, "{} to {}", text, length);
        }
    }
}
