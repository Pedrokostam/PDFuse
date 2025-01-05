use super::{unit::Unit, length::Length};

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ParseResult {
    pub(crate) parsed_value: Length,
    pub(crate) unit: Unit,
    pub(crate) end_position: usize,
}
