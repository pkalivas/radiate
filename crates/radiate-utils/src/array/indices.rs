use crate::Tensor;
use std::ops::Index;

#[inline]
fn flat_index<T, const N: usize>(tensor: &Tensor<T>, index: &[usize; N]) -> usize {
    debug_assert!((0..N).all(|i| index[i] < tensor.shape().dim_at(i)));

    let strides = tensor.strides().as_slice();
    let mut flat = 0;

    // Const-generic N: LLVM typically unrolls for small N.
    for i in 0..N {
        flat += index[i] * strides[i];
    }

    flat
}

impl<T> Index<usize> for Tensor<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T, const N: usize> Index<[usize; N]> for Tensor<T> {
    type Output = T;

    fn index(&self, index: [usize; N]) -> &Self::Output {
        let flat_index = flat_index(self, &index);
        &self.data[flat_index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_index() {
        let tensor = Tensor::new(vec![0; 60], (3, 4, 5));
        let index = [2, 1, 3];
        let flat_idx = flat_index(&tensor, &index);

        let expected_idx = 2 * 20 + 1 * 5 + 3;
        assert_eq!(flat_idx, expected_idx);

        let value = &tensor[index];
        assert_eq!(value, &0);
    }

    #[test]
    fn test_index_1d() {
        let t = Tensor::new(vec![5, 6, 7, 8], 4);
        assert_eq!(t[[0]], 5);
        assert_eq!(t[[3]], 8);
    }

    #[test]
    fn test_index_2d_matches_row_major() {
        // shape (2, 3) row-major => strides [3, 1]
        // data layout:
        // [[0,1,2],
        //  [3,4,5]]
        let t = Tensor::new((0..6).collect::<Vec<i32>>(), (2, 3));
        assert_eq!(t[[0, 0]], 0);
        assert_eq!(t[[0, 2]], 2);
        assert_eq!(t[[1, 0]], 3);
        assert_eq!(t[[1, 2]], 5);
    }

    #[test]
    fn test_index_3d_matches_strides() {
        // shape (2, 3, 4) => strides [12, 4, 1]
        let t = Tensor::new((0..24).collect::<Vec<i32>>(), (2, 3, 4));
        assert_eq!(t.strides().as_slice(), &[12, 4, 1]);

        // flat = a*12 + b*4 + c
        assert_eq!(t[[0, 0, 0]], 0);
        assert_eq!(t[[0, 0, 3]], 3);
        assert_eq!(t[[0, 2, 1]], 0 * 12 + 2 * 4 + 1);
        assert_eq!(t[[1, 0, 0]], 12);
        assert_eq!(t[[1, 2, 3]], 1 * 12 + 2 * 4 + 3);
    }

    #[test]
    fn test_index_4d() {
        // shape (2, 2, 2, 2) => strides [8, 4, 2, 1]
        let t = Tensor::new((0..16).collect::<Vec<i32>>(), (2, 2, 2, 2));
        assert_eq!(t.strides().as_slice(), &[8, 4, 2, 1]);

        // pick a few spots
        assert_eq!(t[[0, 0, 0, 0]], 0);
        assert_eq!(t[[0, 0, 0, 1]], 1);
        assert_eq!(t[[0, 1, 0, 0]], 4);
        assert_eq!(t[[1, 0, 0, 0]], 8);
        assert_eq!(t[[1, 1, 1, 1]], 15);
    }

    #[test]
    fn test_index_rank_gt_5() {
        // shape (1,1,1,1,1,1,4) => rank 7
        // strides should be [4,4,4,4,4,4,1]
        let t = Tensor::new((0..4).collect::<Vec<i32>>(), (1, 1, 1, 1, 1, 1, 4));
        assert_eq!(t.strides().as_slice(), &[4, 4, 4, 4, 4, 4, 1]);

        assert_eq!(t[[0, 0, 0, 0, 0, 0, 0]], 0);
        assert_eq!(t[[0, 0, 0, 0, 0, 0, 3]], 3);
    }

    // This one only works if you use `debug_assert_eq!(tensor.rank(), N)`
    // inside flat_index. cargo test runs with debug assertions by default,
    // but we gate it anyway for clarity.
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic]
    fn test_index_rank_mismatch_panics_in_debug() {
        let t = Tensor::new(vec![0; 6], (2, 3)); // rank 2
        let _ = t[[0, 0, 0]]; // rank 3 index => should trip debug_assert
    }
}
