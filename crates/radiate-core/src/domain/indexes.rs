use crate::random_provider;

pub enum SubsetMode<'a> {
    StratifiedCorrect,
    FastRandom,
    Weighted(&'a [f32]),
    Exclude(&'a [usize]),
    RangeList(&'a [(usize, usize)]),
}

/// * Generates a sorted vector of unique indices for a given size and order, ensuring the specified index is included.
/// * Calls the subset function to get a subset of indices.
/// * Replaces an index in the subset with the specified index if it fits the criteria.
/// * Sorts and returns the result.
pub fn individual_indexes(index: usize, max_index: usize, num_indices: usize) -> Vec<usize> {
    let mut sub_set = subset(max_index, num_indices, SubsetMode::StratifiedCorrect);
    let mut i = 0;
    while i < sub_set.len() && sub_set[i] < index {
        i += 1;
    }
    if i < sub_set.len() {
        sub_set[i] = index;
    }
    let mut result = sub_set.iter().map(|&x| x as usize).collect::<Vec<usize>>();
    result.sort_unstable();
    result
}

/// * Generates a subset of indices of size k from a total of n elements.
/// * Calls the next function to fill the subset.
pub fn subset(max_index: usize, num_indicies: usize, mode: SubsetMode) -> Vec<usize> {
    if max_index < num_indicies {
        panic!("n smaller than k: {} < {}.", max_index, num_indicies);
    }

    match mode {
        SubsetMode::StratifiedCorrect => {
            let mut sub = vec![0; num_indicies];
            next(max_index, &mut sub);
            return sub;
        }
        SubsetMode::FastRandom => {
            let mut sub = vec![0; num_indicies];
            for i in 0..num_indicies {
                sub[i] = random_provider::range(0..max_index);
            }
            return sub;
        }
        SubsetMode::Weighted(weights) => {
            let mut sub = vec![0; num_indicies];
            for i in 0..num_indicies {
                sub[i] = random_provider::weighted_choice(weights);
            }
            return sub;
        }
        SubsetMode::Exclude(exclude) => {
            let mut sub = vec![0; num_indicies];
            for i in 0..num_indicies {
                loop {
                    let index = random_provider::range(0..max_index);
                    if !exclude.contains(&index) {
                        sub[i] = index;
                        break;
                    }
                }
            }
            return sub;
        }
        SubsetMode::RangeList(range_list) => {
            let mut sub = vec![0; num_indicies];
            for i in 0..num_indicies {
                let (start, end) = range_list[i % range_list.len()];
                sub[i] = random_provider::range(start..end);
            }
            return sub;
        }
    }
}

/// * Fills the subset with indices.
/// * If the subset size equals the total number of elements, it fills the subset with sequential indices.
/// * Otherwise, it calls build_subset to generate the subset and invert if necessary.
/// * build_subset Function:
/// * Constructs a subset of indices using a random selection process.
/// * Ensures the subset size and range are valid.
/// * Initializes the subset with evenly spaced indices.
/// * Adjusts the subset by randomly selecting indices and ensuring they are unique.
fn next(max_index: usize, sub_set: &mut [usize]) {
    let k = sub_set.len();
    if k == max_index {
        for i in 0..k {
            sub_set[i] = i;
        }
        return;
    }
    build_subset(max_index, sub_set);
    if k > max_index - k {
        invert(max_index, sub_set);
    }
}

/// * Inverts the subset to ensure all indices are unique and within the specified range.
/// * Uses a helper vector to track used indices and fills the subset with the remaining indices.
fn build_subset(max_index: usize, sub: &mut [usize]) {
    let k = sub.len();
    check_subset(max_index, k);

    for i in 0..k {
        sub[i] = i * max_index / k;
    }

    for _ in 0..k {
        let mut ix;
        let mut l;
        loop {
            ix = random_provider::range(1..max_index);
            l = (ix * k - 1) / max_index;
            if sub[l] < ix {
                break;
            }
        }
        sub[l] += 1;
    }

    let mut ip = 0;
    let mut is_ = k;
    for i in 0..k {
        let m = sub[i];
        sub[i] = 0;
        if m != i * max_index / k {
            ip += 1;
            sub[ip - 1] = m;
        }
    }

    let ihi = ip;
    for i in 1..=ihi {
        ip = ihi + 1 - i;
        let l = 1 + (sub[ip - 1] * k - 1) / max_index;
        let ids = sub[ip - 1] - (l - 1) * max_index / k;
        sub[ip - 1] = 0;
        sub[is_ - 1] = l;
        is_ -= ids;
    }

    for ll in 1..=k {
        let l = k + 1 - ll;
        if sub[l - 1] != 0 {
            let ir = l;
            let m0 = 1 + (sub[l - 1] - 1) * max_index / k;
            let m = sub[l - 1] * max_index / k - m0 + 1;

            let ix = random_provider::range(m0..m0 + m - 1);
            let mut i = l + 1;
            while i <= ir && ix >= sub[i - 1] {
                sub[i - 2] = sub[i - 1];
                i += 1;
            }
            sub[i - 2] = ix;
        }
    }
}

/// * Finds the index of a value in a subset.
/// * Returns the index if found, otherwise returns -1.
fn invert(n: usize, a: &mut [usize]) {
    let k = a.len();
    let mut v = n - 1;
    let j = n - k - 1;
    let mut ac = vec![0; k];
    ac.copy_from_slice(a);

    for i in (0..k).rev() {
        while let Some(_) = index_of(&ac, j, v) {
            v -= 1;
        }
        a[i] = v;
        v -= 1;
    }
}

fn index_of(a: &[usize], start: usize, value: usize) -> Option<usize> {
    for i in (0..=start).rev() {
        if a[i] == value {
            return Some(i);
        }
    }

    None
}

fn check_subset(n: usize, k: usize) {
    if k <= 0 {
        panic!("Subset size smaller or equal to zero: {}", k);
    }
    if n < k {
        panic!("n smaller than k: {} < {}.", n, k);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_random_subset() {
        let n = 50;
        let k = 20;
        let result = subset(n, k, SubsetMode::FastRandom);
        assert_eq!(result.len(), k);
        assert!(result.iter().all(|&x| x < n));
    }

    #[test]
    fn test_weighted_subset() {
        let weights = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let result = subset(weights.len(), 5, SubsetMode::Weighted(&weights));
        assert_eq!(result.len(), 5);
        assert!(result.iter().all(|&x| x < weights.len()));
    }

    #[test]
    fn test_exclude_subset() {
        let n = 10;
        let k = 5;
        let blacklist = vec![2, 3, 4];
        let result = subset(n, k, SubsetMode::Exclude(&blacklist));
        assert_eq!(result.len(), k);
        assert!(result.iter().all(|&x| !blacklist.contains(&x)));
    }

    #[test]
    fn test_range_list_subset() {
        let ranges = vec![(0, 5), (10, 15)];
        let result = subset(20, 6, SubsetMode::RangeList(&ranges));
        assert_eq!(result.len(), 6);
        assert!(
            result
                .iter()
                .all(|&x| (0..5).contains(&x) || (10..15).contains(&x))
        );
    }

    #[test]
    fn test_individual_indexes_includes_index() {
        let result = individual_indexes(7, 20, 5);
        assert_eq!(result.len(), 5);
        assert!(result.contains(&7));
        let mut sorted = result.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, result); // must be sorted
    }
}
