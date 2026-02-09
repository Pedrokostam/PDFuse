use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct DisplayableRange(std::ops::Range<usize>);
impl DisplayableRange {
    pub fn len(&self) -> usize {
        let diff = self.0.end - self.0.start;
        match diff {
            0 | 1 => 1,
            d => d,
        }
    }

    pub fn first(&self) -> usize {
        self.0.start
    }

    pub fn into_vec(self) -> Vec<usize> {
        self.0.collect()
    }
}

impl Display for DisplayableRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.len() == 1 {
            // One if actually an invalid range in this case
            // All ranges should be open on the end (be exclusive)
            write!(f, "{}", self.0.start)
        } else {
            // Range covers more than 1 page
            write!(f, "{}…{}", self.0.start, self.0.end - 1)
        }
    }
}

impl From<std::ops::Range<usize>> for DisplayableRange {
    fn from(value: std::ops::Range<usize>) -> Self {
        DisplayableRange(value)
    }
}

impl From<usize> for DisplayableRange {
    fn from(value: usize) -> Self {
        DisplayableRange(value..(value + 1))
    }
}

impl From<Vec<usize>> for DisplayableRange {
    fn from(value: Vec<usize>) -> Self {
        DisplayableRange::from(value.as_slice())
    }
}

impl From<&[usize]> for DisplayableRange {
    fn from(value: &[usize]) -> Self {
        // Underlying range is closed on the front and open on the end
        // Since `value` has all discrete pages
        // The range has to go 1 "page" further than what is in `value`
        assert!(!value.is_empty(), "No page indices provided!");
        let start = value[0];
        let end = 1 + value
            .last()
            .expect("We already checked that value has elements");
        // if value.len() == 1 {
        //     return DisplayableRange(value[0]..value[0] + 1);
        // }
        // let mut last_val: usize = value[0];
        // for v in value.iter().skip(1) {
        //     assert!(v - 1 == last_val);
        //     last_val = *v;
        // }
        DisplayableRange(start..end)
    }
}

impl From<DisplayableRange> for Vec<usize> {
    fn from(value: DisplayableRange) -> Self {
        value.0.collect()
    }
}
