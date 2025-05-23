use crate::errors::UnitParseError;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Unit {
    Meter,
    Millimeter,
    Inch,
    Point,
    Centimeter,
}

impl Unit {
    pub fn unit_symbol(&self) -> &str {
        match self {
            Unit::Meter => "m",
            Unit::Millimeter => "mm",
            Unit::Inch => "in",
            Unit::Point => "pt",
            Unit::Centimeter => "cm",
        }
    }
    pub const DEFAULT_UNIT: Unit = Unit::Centimeter;
    fn from_string_impl(text: &str) -> Result<Self, UnitParseError> {
        match text {
            "m" | "meter" | "meters" => Ok(Unit::Meter),
            "mm" | "milli" | "millis" | "millimeters" | "millimeter" => Ok(Unit::Millimeter),
            "cm" | "centimeters" | "centimeter" => Ok(Unit::Centimeter),
            "in" | "inch" | "inches" => Ok(Unit::Inch),
            "pt" | "point" | "points" => Ok(Unit::Point),
            "" => Err(UnitParseError::NoUnitSpecified),
            _ => Err(UnitParseError::UnrecognizedUnit(text.to_owned())),
        }
    }
    pub fn from_string(text: &str) -> Result<Self, UnitParseError> {
        let trim = text.trim();
        Self::from_string_impl(trim).or_else(|_| Self::from_string_impl(&trim.to_ascii_lowercase()))
    }
    /// mm / m
    const MM_OVER_M: f64 = 1000.0;
    /// cm / m
    const CM_OVER_M: f64 = 100.0;
    /// in / m
    const IN_OVER_M: f64 = 39.37;
    /// mm / cm
    const MM_OVER_CM: f64 = 10.0;
    /// cm / in
    const CM_OVER_IN: f64 = 2.54;
    /// mm / in
    const MM_OVER_IN: f64 = 25.4;
    /// pt / in
    const PT_OVER_IN: f64 = 72.0;

    pub fn get_multiplier(from: Unit, to: Unit) -> f64 {
        Self::change_unit(1.0, from, to)
    }
    pub fn change_unit(value: f64, from: Unit, to: Unit) -> f64 {
        let safe_margin = match to {
            Unit::Meter => 1000.0,
            Unit::Millimeter => 10.0,
            Unit::Inch => 100.0,
            Unit::Point => 10.0,
            Unit::Centimeter => 100.0,
        };
        let safe = value * safe_margin;
        let conv = match (from, to) {
            (Unit::Millimeter, Unit::Millimeter) => safe,
            (Unit::Inch, Unit::Inch) => safe,
            (Unit::Point, Unit::Point) => safe,
            (Unit::Centimeter, Unit::Centimeter) => safe,
            (Unit::Meter, Unit::Meter) => safe,

            (Unit::Millimeter, Unit::Inch) => safe / Self::MM_OVER_IN,
            (Unit::Millimeter, Unit::Point) => safe * Self::PT_OVER_IN / Self::MM_OVER_IN,
            (Unit::Millimeter, Unit::Centimeter) => safe / Self::MM_OVER_CM,
            (Unit::Millimeter, Unit::Meter) => safe / Self::MM_OVER_M,

            (Unit::Inch, Unit::Millimeter) => safe * Self::MM_OVER_IN,
            (Unit::Inch, Unit::Point) => safe * Self::PT_OVER_IN,
            (Unit::Inch, Unit::Centimeter) => safe * Self::CM_OVER_IN,
            (Unit::Inch, Unit::Meter) => safe / Self::IN_OVER_M,

            (Unit::Point, Unit::Millimeter) => safe * Self::MM_OVER_IN / Self::PT_OVER_IN,
            (Unit::Point, Unit::Inch) => safe / Self::PT_OVER_IN,
            (Unit::Point, Unit::Centimeter) => safe * Self::CM_OVER_IN / Self::PT_OVER_IN,
            (Unit::Point, Unit::Meter) => safe / Self::IN_OVER_M / Self::PT_OVER_IN,

            (Unit::Centimeter, Unit::Millimeter) => safe * Self::MM_OVER_CM,
            (Unit::Centimeter, Unit::Inch) => safe / Self::CM_OVER_IN,
            (Unit::Centimeter, Unit::Point) => safe * Self::PT_OVER_IN / Self::CM_OVER_IN,
            (Unit::Centimeter, Unit::Meter) => safe / Self::CM_OVER_M,

            (Unit::Meter, Unit::Millimeter) => safe * Self::MM_OVER_M,
            (Unit::Meter, Unit::Inch) => safe * Self::IN_OVER_M,
            (Unit::Meter, Unit::Point) => safe * Self::PT_OVER_IN * Self::IN_OVER_M,
            (Unit::Meter, Unit::Centimeter) => safe * Self::CM_OVER_M,
        };
        conv.round() / safe_margin
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parsing_unit() {
        let vals = vec![
            (
                vec!["mm", "milli", "millis", "millimeter", "millimeters"],
                Unit::Millimeter,
            ),
            (vec!["cm", "centimeters", "centimeter"], Unit::Centimeter),
            (vec!["m", "meters", "meter"], Unit::Meter),
            (vec!["in", "inch", "inches"], Unit::Inch),
            (vec!["pt", "point", "points"], Unit::Point),
        ];
        for (texts, unit) in vals {
            for text in texts {
                let target = Ok(unit);
                assert_eq!(Unit::from_string(text), target, "{text}");
                let pad = format!("  {text}  ");
                assert_eq!(Unit::from_string(&pad), target, "{}", &pad);
                let upper = pad.to_uppercase();
                assert_eq!(Unit::from_string(&upper), target, "{}", &upper);
            }
        }
    }
}
