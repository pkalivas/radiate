#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{ops::Range, sync::Arc};

#[derive(Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RangeLookup<T> {
    bounds: Arc<[Range<T>]>,
    index_to_bounds_map: Arc<[usize]>,
}

impl<T> RangeLookup<T> {
    pub fn new(bounds: Vec<Range<T>>, index_to_bounds_map: Vec<usize>) -> Self {
        Self {
            bounds: Arc::from(bounds),
            index_to_bounds_map: Arc::from(index_to_bounds_map),
        }
    }

    pub fn get(&self, index: usize) -> Option<&Range<T>> {
        self.index_to_bounds_map
            .get(index)
            .and_then(|&bounds_index| self.bounds.get(bounds_index))
    }
}
