use std::ops::Range;

use crate::Op;

impl<T> Op<T> {
    pub fn var(index: usize) -> Self {
        let name = radiate_core::intern!(format!("X{}", index));
        Op::Var(name, index)
    }

    pub fn vars(range: Range<usize>) -> Vec<Self> {
        range.map(Op::var).collect()
    }
}
