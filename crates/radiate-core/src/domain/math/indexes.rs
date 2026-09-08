use crate::{RdRand, random_provider};

/// * Generates a sorted vector of unique indices for a given size and order, ensuring the specified index is included.
/// * Calls the subset function to get a subset of indices.
/// * Replaces an index in the subset with the specified index if it fits the criteria.
/// * Sorts and returns the result.
pub fn create_subset_inclusive(index: usize, max_index: usize, num_indices: usize) -> Vec<usize> {
    let mut scratch = vec![0; num_indices];
    fill_subset_inclusive(index, max_index, &mut scratch);
    scratch
}

#[inline]
pub fn fill_subset_inclusive(index: usize, max_index: usize, buff: &mut [usize]) {
    fill_subset(max_index, buff);

    let mut i = 0;
    while i < buff.len() - 1 && buff[i] < index {
        i += 1;
    }

    buff[i] = index;
    buff.sort_unstable();
}

#[inline]
pub fn subset(num_indices: usize, max_index: usize) -> Vec<usize> {
    let mut buffer = vec![0; num_indices];
    fill_subset(max_index, &mut buffer);
    buffer
}

/// * Generates a subset of indices of size k from a total of n elements.
/// * Calls the next function to fill the subset.
#[inline]
pub fn fill_subset(max_index: usize, buffer: &mut [usize]) {
    random_provider::with_rng(|rand| {
        next(max_index, buffer, rand);
    })
}

/// * Fills the subset with indices.
/// * If the subset size equals the total number of elements, it fills the subset with sequential indices.
/// * Otherwise, it calls build_subset to generate the subset and invert if necessary.
/// * build_subset Function:
/// * Constructs a subset of indices using a random selection process.
/// * Ensures the subset size and range are valid.
/// * Initializes the subset with evenly spaced indices.
/// * Adjusts the subset by randomly selecting indices and ensuring they are unique
#[inline]
fn next(max_index: usize, sub_set: &mut [usize], rand: &mut RdRand<'_>) {
    let k = sub_set.len();
    if k == max_index {
        for (i, item) in sub_set.iter_mut().enumerate() {
            *item = i;
        }
        return;
    }

    if k > max_index - k {
        // build_subset's internal bin math requires k' <= n - k' to hold.
        // Build only the complementary (n - k)-sized subset into the front
        // of the buffer, then invert() expands it into the full result
        let kp = max_index - k;
        build_subset(max_index, &mut sub_set[..kp], rand);
        invert(max_index, sub_set);
    } else {
        build_subset(max_index, sub_set, rand);
    }
}

/// * Inverts the subset to ensure all indices are unique and within the specified range.
/// * Uses a helper vector to track used indices and fills the subset with the remaining indices.
#[inline]
fn build_subset(max_index: usize, sub: &mut [usize], rand: &mut RdRand<'_>) {
    let k = sub.len();
    check_subset(max_index, k);

    for (i, item) in sub.iter_mut().enumerate() {
        *item = i * max_index / k;
    }

    for _ in 0..k {
        let mut ix;
        let mut l;
        loop {
            ix = 1 + rand.range(0..max_index);
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

    let mut r = 0usize;
    let mut m0 = 0usize;
    let mut m = 0usize;

    for ll in 1..=k {
        let l = k + 1 - ll;

        if sub[l - 1] != 0 {
            r = l;
            m0 = 1 + (sub[l - 1] - 1) * max_index / k;
            m = sub[l - 1] * max_index / k - m0 + 1;
        }

        let ix = m0 + rand.range(0..m);
        let mut i = l + 1;
        let mut x = ix;
        while i <= r && x >= sub[i - 1] {
            x += 1;
            sub[i - 2] = sub[i - 1];
            i += 1;
        }
        sub[i - 2] = x;
        m -= 1;
    }

    for item in sub.iter_mut() {
        *item -= 1;
    }
}

/// * Finds the index of a value in a subset.
/// * Returns the index if found, otherwise returns -1.
#[inline]
fn invert(n: usize, a: &mut [usize]) {
    let k = a.len();
    let mut v = n - 1;
    let j = n - k - 1;
    let mut ac = vec![0; k];
    ac.copy_from_slice(a);

    for i in (0..k).rev() {
        while index_of(&ac, j, v).is_some() {
            v = v.saturating_sub(1);
        }
        a[i] = v;
        v = v.saturating_sub(1);
    }
}

#[inline]
fn index_of(a: &[usize], start: usize, value: usize) -> Option<usize> {
    (0..=start).rev().find(|&i| a[i] == value)
}

#[inline]
fn check_subset(n: usize, k: usize) {
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
        let mut result = vec![0; k];
        fill_subset(k, &mut result);
        assert_eq!(result.len(), k);
        assert!(result.iter().all(|&x| x < n));
    }

    #[test]
    fn test_individual_indexes_includes_index() {
        let result = create_subset_inclusive(7, 20, 5);
        assert_eq!(result.len(), 5);
        assert!(result.contains(&7));
        let mut sorted = result.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, result); // must be sorted
    }

    #[test]
    fn individual_indexes_always_includes_index_even_at_the_high_end() {
        // Deliberately picks an index near the top of the range so the
        // "index larger than every drawn value" path gets exercised —
        // the bug this test targets only manifests there.
        for _ in 0..1_000 {
            let (max_index, num_indices, index) = (20, 5, 19);
            let mut buf = vec![0usize; num_indices];
            fill_subset_inclusive(index, max_index, &mut buf);
            assert!(
                buf.contains(&index),
                "index {} missing from {:?}",
                index,
                buf
            );

            let mut sorted = buf.clone();
            sorted.sort_unstable();
            assert_eq!(sorted, buf, "not sorted: {:?}", buf);
        }
    }
}
