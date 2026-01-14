use std::ops::{Index, IndexMut};

use crate::{Layout, LayoutView, LayoutViewMut};

impl<T> Index<usize> for Layout<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T> IndexMut<usize> for Layout<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<T> Index<&[usize]> for Layout<T> {
    type Output = T;

    fn index(&self, index: &[usize]) -> &Self::Output {
        self.get_nd(index)
    }
}

impl<T> IndexMut<&[usize]> for Layout<T> {
    fn index_mut(&mut self, index: &[usize]) -> &mut Self::Output {
        self.get_nd_mut(index)
    }
}

impl<T, const N: usize> Index<[usize; N]> for Layout<T> {
    type Output = T;

    fn index(&self, index: [usize; N]) -> &Self::Output {
        self.get_nd(&index)
    }
}

impl<T, const N: usize> IndexMut<[usize; N]> for Layout<T> {
    fn index_mut(&mut self, index: [usize; N]) -> &mut Self::Output {
        self.get_nd_mut(&index)
    }
}

impl<T> Index<(usize, usize)> for Layout<T> {
    type Output = T;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let flat_index = self.flat_index(&[index.0, index.1]);
        &self.data[flat_index]
    }
}

impl<T> IndexMut<(usize, usize)> for Layout<T> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let flat_index = self.flat_index(&[index.0, index.1]);
        &mut self.data[flat_index]
    }
}

impl<T> Index<(usize, usize, usize)> for Layout<T> {
    type Output = T;

    fn index(&self, index: (usize, usize, usize)) -> &Self::Output {
        let flat_index = self.flat_index(&[index.0, index.1, index.2]);
        &self.data[flat_index]
    }
}

impl<T> IndexMut<(usize, usize, usize)> for Layout<T> {
    fn index_mut(&mut self, index: (usize, usize, usize)) -> &mut Self::Output {
        let flat_index = self.flat_index(&[index.0, index.1, index.2]);
        &mut self.data[flat_index]
    }
}

impl<T> Index<(usize, usize, usize, usize)> for Layout<T> {
    type Output = T;

    fn index(&self, index: (usize, usize, usize, usize)) -> &Self::Output {
        let flat_index = self.flat_index(&[index.0, index.1, index.2, index.3]);
        &self.data[flat_index]
    }
}

impl<T> IndexMut<(usize, usize, usize, usize)> for Layout<T> {
    fn index_mut(&mut self, index: (usize, usize, usize, usize)) -> &mut Self::Output {
        let flat_index = self.flat_index(&[index.0, index.1, index.2, index.3]);
        &mut self.data[flat_index]
    }
}

impl<T> Index<(usize, usize, usize, usize, usize)> for Layout<T> {
    type Output = T;

    fn index(&self, index: (usize, usize, usize, usize, usize)) -> &Self::Output {
        let flat_index = self.flat_index(&[index.0, index.1, index.2, index.3, index.4]);
        &self.data[flat_index]
    }
}

impl<T> IndexMut<(usize, usize, usize, usize, usize)> for Layout<T> {
    fn index_mut(&mut self, index: (usize, usize, usize, usize, usize)) -> &mut Self::Output {
        let flat_index = self.flat_index(&[index.0, index.1, index.2, index.3, index.4]);
        &mut self.data[flat_index]
    }
}

// --- Index impls for LayoutView ---

impl<'a, T> Index<&[usize]> for LayoutView<'a, T> {
    type Output = T;

    fn index(&self, index: &[usize]) -> &Self::Output {
        self.get_nd(index)
    }
}

impl<'a, T, const N: usize> Index<[usize; N]> for LayoutView<'a, T> {
    type Output = T;

    fn index(&self, index: [usize; N]) -> &Self::Output {
        self.get_nd(&index)
    }
}

impl<'a, T> Index<(usize, usize)> for LayoutView<'a, T> {
    type Output = T;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        self.get_nd(&[index.0, index.1])
    }
}

impl<'a, T> Index<(usize, usize, usize)> for LayoutView<'a, T> {
    type Output = T;

    fn index(&self, index: (usize, usize, usize)) -> &Self::Output {
        self.get_nd(&[index.0, index.1, index.2])
    }
}

impl<'a, T> Index<(usize, usize, usize, usize)> for LayoutView<'a, T> {
    type Output = T;

    fn index(&self, index: (usize, usize, usize, usize)) -> &Self::Output {
        self.get_nd(&[index.0, index.1, index.2, index.3])
    }
}

impl<'a, T> Index<(usize, usize, usize, usize, usize)> for LayoutView<'a, T> {
    type Output = T;

    fn index(&self, index: (usize, usize, usize, usize, usize)) -> &Self::Output {
        self.get_nd(&[index.0, index.1, index.2, index.3, index.4])
    }
}

// --- Index impls for LayoutViewMut ---
impl<'a, T> Index<&[usize]> for LayoutViewMut<'a, T> {
    type Output = T;

    fn index(&self, index: &[usize]) -> &Self::Output {
        self.get_nd(index)
    }
}

impl<'a, T> IndexMut<&[usize]> for LayoutViewMut<'a, T> {
    fn index_mut(&mut self, index: &[usize]) -> &mut Self::Output {
        self.get_nd_mut(index)
    }
}

impl<'a, T, const N: usize> Index<[usize; N]> for LayoutViewMut<'a, T> {
    type Output = T;

    fn index(&self, index: [usize; N]) -> &Self::Output {
        self.get_nd(&index)
    }
}

impl<'a, T, const N: usize> IndexMut<[usize; N]> for LayoutViewMut<'a, T> {
    fn index_mut(&mut self, index: [usize; N]) -> &mut Self::Output {
        self.get_nd_mut(&index)
    }
}

impl<'a, T> Index<(usize, usize)> for LayoutViewMut<'a, T> {
    type Output = T;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        self.get_nd(&[index.0, index.1])
    }
}

impl<'a, T> IndexMut<(usize, usize)> for LayoutViewMut<'a, T> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        self.get_nd_mut(&[index.0, index.1])
    }
}

impl<'a, T> Index<(usize, usize, usize)> for LayoutViewMut<'a, T> {
    type Output = T;

    fn index(&self, index: (usize, usize, usize)) -> &Self::Output {
        self.get_nd(&[index.0, index.1, index.2])
    }
}

impl<'a, T> IndexMut<(usize, usize, usize)> for LayoutViewMut<'a, T> {
    fn index_mut(&mut self, index: (usize, usize, usize)) -> &mut Self::Output {
        self.get_nd_mut(&[index.0, index.1, index.2])
    }
}

impl<'a, T> Index<(usize, usize, usize, usize)> for LayoutViewMut<'a, T> {
    type Output = T;

    fn index(&self, index: (usize, usize, usize, usize)) -> &Self::Output {
        self.get_nd(&[index.0, index.1, index.2, index.3])
    }
}

impl<'a, T> IndexMut<(usize, usize, usize, usize)> for LayoutViewMut<'a, T> {
    fn index_mut(&mut self, index: (usize, usize, usize, usize)) -> &mut Self::Output {
        self.get_nd_mut(&[index.0, index.1, index.2, index.3])
    }
}

impl<'a, T> Index<(usize, usize, usize, usize, usize)> for LayoutViewMut<'a, T> {
    type Output = T;

    fn index(&self, index: (usize, usize, usize, usize, usize)) -> &Self::Output {
        self.get_nd(&[index.0, index.1, index.2, index.3, index.4])
    }
}

impl<'a, T> IndexMut<(usize, usize, usize, usize, usize)> for LayoutViewMut<'a, T> {
    fn index_mut(&mut self, index: (usize, usize, usize, usize, usize)) -> &mut Self::Output {
        self.get_nd_mut(&[index.0, index.1, index.2, index.3, index.4])
    }
}
