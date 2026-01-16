/// Errors returned by fallible `Layout` constructors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TensorError {
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

impl std::fmt::Display for TensorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TensorError::ShapeOverflow { dims } => {
                write!(f, "shape overflow for dimensions {:?}", dims)
            }
            TensorError::LenMismatch { len, expected } => {
                write!(f, "length mismatch: got {}, expected {}", len, expected)
            }
            TensorError::NonContiguousRow { axis } => {
                write!(f, "requested non-contiguous row slice along axis {}", axis)
            }
            TensorError::RankMismatch { got, expected } => {
                write!(f, "rank mismatch: got {}, expected {}", got, expected)
            }
        }
    }
}
