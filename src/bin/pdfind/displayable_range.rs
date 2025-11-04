use std::fmt::Display;

pub struct DisplayableRange(std::ops::Range<usize>);

impl DisplayableRange {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn first(&self) -> usize {
        self.0.start
    }

    pub fn collect(&self) -> Vec<usize> {
        self.0.clone().collect()
    }
}

impl Display for DisplayableRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.len() == 1 {
            write!(f, "{}", self.0.start)
        } else {
            write!(f, "{}…{}", self.0.start, self.0.end)
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
        assert!(!value.is_empty(), "Page indices are not consecutive!");
        if value.len() == 1 {
            return DisplayableRange(value[0]..value[0] + 1);
        }
        let mut last_val: usize = value[0];
        for v in value.iter().skip(1) {
            assert!(v - 1 == last_val);
            last_val = *v;
        }
        DisplayableRange(value[0]..last_val)
    }
}

impl From<DisplayableRange> for Vec<usize> {
    fn from(value: DisplayableRange) -> Self {
        value.0.collect()
    }
}
