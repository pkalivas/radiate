use crate::Primitive;

pub struct MinMax<T: Primitive> {
    min: T,
    max: T,
}

impl<T: Primitive> MinMax<T> {
    pub fn min(&self) -> T {
        self.min
    }

    pub fn max(&self) -> T {
        self.max
    }

    pub fn min_max(&self) -> (T, T) {
        (self.min, self.max)
    }

    pub fn range(&self) -> T {
        self.max - self.min
    }

    #[inline]
    pub fn add(&mut self, value: &T) {
        if value < &self.min {
            self.min = *value;
        }

        if value > &self.max {
            self.max = *value;
        }
    }
}

impl<T: Primitive> Default for MinMax<T> {
    fn default() -> Self {
        MinMax {
            min: T::MAX,
            max: T::MIN,
        }
    }
}

impl<T> FromIterator<T> for MinMax<T>
where
    T: Primitive,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut min_max = MinMax::default();

        for value in iter {
            min_max.add(&value);
        }

        min_max
    }
}

impl<'a, T> FromIterator<&'a T> for MinMax<T>
where
    T: Primitive,
{
    fn from_iter<I: IntoIterator<Item = &'a T>>(iter: I) -> Self {
        let mut min_max = MinMax::default();

        for value in iter {
            min_max.add(value);
        }

        min_max
    }
}
