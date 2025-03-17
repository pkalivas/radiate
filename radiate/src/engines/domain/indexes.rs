use crate::random_provider;

/// * Generates a sorted vector of unique indices for a given size and order, ensuring the specified index is included.
/// * Calls the subset function to get a subset of indices.
/// * Replaces an index in the subset with the specified index if it fits the criteria.
/// * Sorts and returns the result.
pub fn individual_indexes(index: usize, size: usize, order: usize) -> Vec<usize> {
    let mut sub_set = subset(size, order);
    let mut i = 0;
    while i < sub_set.len() && sub_set[i] < index as i32 {
        i += 1;
    }
    if i < sub_set.len() {
        sub_set[i] = index as i32;
    }
    let mut result = sub_set.iter().map(|&x| x as usize).collect::<Vec<usize>>();
    result.sort_unstable();
    result
}

/// * Generates a subset of indices of size k from a total of n elements.
/// * Calls the next function to fill the subset.
pub fn subset(n: usize, k: usize) -> Vec<i32> {
    if n < k {
        panic!("n smaller than k: {} < {}.", n, k);
    }
    let mut sub = vec![0; k];
    next(n as i32, &mut sub);
    sub
}

/// * Fills the subset with indices.
/// * If the subset size equals the total number of elements, it fills the subset with sequential indices.
/// * Otherwise, it calls build_subset to generate the subset and invert if necessary.
/// * build_subset Function:
/// * Constructs a subset of indices using a random selection process.
/// * Ensures the subset size and range are valid.
/// * Initializes the subset with evenly spaced indices.
/// * Adjusts the subset by randomly selecting indices and ensuring they are unique.
fn next(num: i32, a: &mut [i32]) {
    let k = a.len() as i32;
    if k == num {
        for i in 0..k {
            a[i as usize] = i;
        }
        return;
    }
    build_subset(num, a);
    if k > num - k {
        invert(num, a);
    }
}

/// * Inverts the subset to ensure all indices are unique and within the specified range.
/// * Uses a helper vector to track used indices and fills the subset with the remaining indices.
fn build_subset(n: i32, sub: &mut [i32]) {
    let k = sub.len() as i32;
    check_subset(n, k);

    for i in 0..k {
        sub[i as usize] = i * n / k;
    }

    for _ in 0..k {
        let mut ix;
        let mut l;
        loop {
            ix = random_provider::range(1..n);
            l = (ix * k - 1) / n;
            if sub[l as usize] < ix {
                break;
            }
        }
        sub[l as usize] += 1;
    }

    let mut ip = 0;
    let mut is_ = k;
    for i in 0..k {
        let m = sub[i as usize];
        sub[i as usize] = 0;
        if m != i * n / k {
            ip += 1;
            sub[ip as usize - 1] = m;
        }
    }

    let ihi = ip;
    for i in 1..=ihi {
        ip = ihi + 1 - i;
        let l = 1 + (sub[ip as usize - 1] * k - 1) / n;
        let ids = sub[ip as usize - 1] - (l - 1) * n / k;
        sub[ip as usize - 1] = 0;
        sub[is_ as usize - 1] = l;
        is_ -= ids;
    }

    for ll in 1..=k {
        let l = k + 1 - ll;
        if sub[l as usize - 1] != 0 {
            let ir = l;
            let m0 = 1 + (sub[l as usize - 1] - 1) * n / k;
            let m = sub[l as usize - 1] * n / k - m0 + 1;
            let ix = random_provider::range(m0..m0 + m - 1);
            let mut i = l + 1;
            while i <= ir && ix >= sub[i as usize - 1] {
                sub[i as usize - 2] = sub[i as usize - 1];
                i += 1;
            }
            sub[i as usize - 2] = ix;
        }
    }
}

/// * Finds the index of a value in a subset.
/// * Returns the index if found, otherwise returns -1.
fn invert(n: i32, a: &mut [i32]) {
    let k = a.len() as i32;
    let mut v = n - 1;
    let j = n - k - 1;
    let mut ac = vec![0; k as usize];
    ac.copy_from_slice(a);

    for i in (0..k).rev() {
        while index_of(&ac, j, v) == -1 {
            v -= 1;
        }
        a[i as usize] = v;
        v -= 1;
    }
}

fn index_of(a: &[i32], start: i32, value: i32) -> i32 {
    for i in (0..=start).rev() {
        if a[i as usize] == value {
            return i;
        }
    }
    -1
}

fn check_subset(n: i32, k: i32) {
    if k <= 0 {
        panic!("Subset size smaller or equal to zero: {}", k);
    }
    if n < k {
        panic!("n smaller than k: {} < {}.", n, k);
    }
}
