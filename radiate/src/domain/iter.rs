pub trait SortableIter: Iterator + Sized {
    /// Sorts the elements of the iterator in ascending order.
    fn sort(self) -> impl Iterator<Item = Self::Item>
    where
        Self: Iterator,
        Self::Item: Ord,
    {
        let mut vec: Vec<Self::Item> = self.collect();
        vec.sort();
        vec.into_iter()
    }

    fn sort_by<F>(self, compare: F) -> impl Iterator<Item = Self::Item>
    where
        Self: Iterator,
        Self::Item: Ord,
        F: Fn(&Self::Item, &Self::Item) -> std::cmp::Ordering,
    {
        let mut vec: Vec<Self::Item> = self.collect();
        vec.sort_by(compare);
        vec.into_iter()
    }

    fn sort_by_key<K, F>(self, key: F) -> impl Iterator<Item = Self::Item>
    where
        Self: Iterator,
        Self::Item: Ord,
        F: Fn(&Self::Item) -> K,
        K: Ord,
    {
        let mut vec: Vec<Self::Item> = self.collect();
        vec.sort_by_key(key);
        vec.into_iter()
    }

    fn sort_descending(self) -> impl Iterator<Item = Self::Item>
    where
        Self: Iterator,
        Self::Item: Ord,
    {
        let mut vec: Vec<Self::Item> = self.collect();
        vec.sort_by(|a, b| b.cmp(a));
        vec.into_iter()
    }
}

impl<I> SortableIter for I where I: Iterator {}

#[cfg(test)]
mod tests {
    use super::SortableIter;

    #[test]
    fn test_sorted() {
        let vec = vec![3, 1, 4, 1, 5, 9];
        let sorted_vec: Vec<_> = vec.iter().sort().cloned().collect();
        assert_eq!(sorted_vec, vec![1, 1, 3, 4, 5, 9]);

        let empty_vec: Vec<i32> = vec![];
        let sorted_empty: Vec<_> = empty_vec.iter().sort().collect();
        assert!(sorted_empty.is_empty());
    }
}
