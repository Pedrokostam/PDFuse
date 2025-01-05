use std::fmt::Display;

use super::{CustomSize, Length};

pub trait Size:Display{
    fn transposed(&self) -> Self;
    fn transpose(&mut self);
    /// Returns a tuple of UnitDistance (horizontal, vertical)
    fn to_custom_size(&self) -> CustomSize;
    fn horizontal(&self) -> Length;
    fn vertical(&self) -> Length;
    /// Returns the scale. If the other size is multiplied by it, it will fit in the checking size. Scaling is uniform.
    fn fit_size(&self, other_size:&CustomSize ) ->f64;
}
