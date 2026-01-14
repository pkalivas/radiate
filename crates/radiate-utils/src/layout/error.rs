/// Errors returned by fallible `Layout` constructors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError {
    ShapeOverflow {
        dims: Vec<usize>,
    },
    LenMismatch {
        len: usize,
        expected: usize,
    },

    /// Requested a contiguous row slice along an axis that is not contiguous.
    NonContiguousRow {
        axis: usize,
    },

    /// Rank mismatch for an operation.
    RankMismatch {
        got: usize,
        expected: usize,
    },
}
