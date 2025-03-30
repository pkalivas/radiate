impl<I> IterSortExt for I where I: Iterator {}

pub trait IterSortExt: Iterator + Sized {
    /// Sorts the elements of the iterator in ascending order.
    fn sort(self) -> impl Iterator<Item = Self::Item>
    where
        Self::Item: Ord,
    {
        let mut vec: Vec<Self::Item> = self.collect();
        vec.sort();
        vec.into_iter()
    }

    fn sort_by<F>(self, compare: F) -> impl Iterator<Item = Self::Item>
    where
        F: Fn(&Self::Item, &Self::Item) -> std::cmp::Ordering,
    {
        let mut vec: Vec<Self::Item> = self.collect();
        vec.sort_by(compare);
        vec.into_iter()
    }

    fn sort_by_key<K, F>(self, key: F) -> impl Iterator<Item = Self::Item>
    where
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
        Self::Item: Ord,
    {
        let mut vec: Vec<Self::Item> = self.collect();
        vec.sort_by(|a, b| b.cmp(a));
        vec.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::IterSortExt;

    #[test]
    fn test_sorted() {
        let vec = vec![3, 1, 4, 1, 5, 9];
        let sorted_vec: Vec<_> = vec.iter().sort().cloned().collect();
        assert_eq!(sorted_vec, vec![1, 1, 3, 4, 5, 9]);

        let empty_vec: Vec<i32> = vec![];
        let sorted_empty: Vec<_> = empty_vec.iter().sort().collect();
        assert!(sorted_empty.is_empty());
    }

    #[test]
    fn test_sorted_by() {
        let vec = vec![3, 1, 4, 1, 5, 9];
        let sorted_vec: Vec<_> = vec.iter().sort_by(|a, b| b.cmp(a)).cloned().collect();
        assert_eq!(sorted_vec, vec![9, 5, 4, 3, 1, 1]);

        let empty_vec: Vec<i32> = vec![];
        let sorted_empty: Vec<_> = empty_vec
            .iter()
            .sort_by(|_, _| std::cmp::Ordering::Equal)
            .collect();
        assert!(sorted_empty.is_empty());
    }

    #[test]
    fn test_sorted_by_key() {
        let vec = vec![("apple", 3), ("banana", 1), ("cherry", 2)];
        let sorted_vec: Vec<_> = vec.iter().sort_by_key(|&(_, v)| *v).cloned().collect();
        assert_eq!(sorted_vec, vec![("banana", 1), ("cherry", 2), ("apple", 3)]);

        let empty_vec: Vec<(&str, i32)> = vec![];
        let sorted_empty: Vec<_> = empty_vec.iter().sort_by_key(|&(_, v)| *v).collect();
        assert!(sorted_empty.is_empty());
    }

    #[test]
    fn test_sorted_descending() {
        let vec = vec![3, 1, 4, 1, 5, 9];
        let sorted_vec: Vec<_> = vec.iter().sort_descending().cloned().collect();
        assert_eq!(sorted_vec, vec![9, 5, 4, 3, 1, 1]);

        let empty_vec: Vec<i32> = vec![];
        let sorted_empty: Vec<_> = empty_vec.iter().sort_descending().collect();
        assert!(sorted_empty.is_empty());
    }
}
