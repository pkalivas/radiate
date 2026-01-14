use crate::Shape;

/// Iterator over all N-D indices for a given shape.
/// This iterator allocates a `Vec<usize>` per item; it's meant for debugging and simple generic algorithms.
pub struct Indices {
    shape: Shape,
    cur: Vec<usize>,
    done: bool,
}

impl Indices {
    pub fn new(shape: Shape) -> Self {
        let rank = shape.rank();
        Self {
            shape,
            cur: vec![0usize; rank],
            done: rank == 0,
        }
    }
}

impl Iterator for Indices {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let out = self.cur.clone();
        // increment like an odometer from the last axis
        for ax in (0..self.shape.rank()).rev() {
            let dim = self.shape.dim_at(ax).max(1);
            self.cur[ax] += 1;
            if self.cur[ax] < dim {
                return Some(out);
            }
            self.cur[ax] = 0;
            if ax == 0 {
                self.done = true;
            }
        }
        Some(out)
    }
}
