use rand::{rngs::ThreadRng, Rng};

pub fn individual_indexes(
    random: &mut ThreadRng,
    index: usize,
    size: usize,
    order: usize,
) -> Vec<usize> {
    let mut sub_set = subset(size as usize, order as usize, random);
    let mut i = 0;
    while sub_set[i] < index as i32 && i < sub_set.len() - 1 {
        i += 1;
    }

    sub_set[i] = index as i32;
    let mut result = sub_set.iter().map(|x| *x as usize).collect::<Vec<usize>>();
    result.sort();
    result
}

pub fn subset(n: usize, k: usize, random: &mut ThreadRng) -> Vec<i32> {
    if k <= 0 {
        panic!("Subset size smaller or equal zero: {}", k);
    }

    if n < k {
        panic!("n smaller than k: {} < {}.", n, k);
    }

    let mut sub = vec![0; k as usize];
    next(n as i32, &mut sub, random);
    sub
}

fn next(num: i32, a: &mut Vec<i32>, random: &mut ThreadRng) {
    let k = a.len() as i32;

    if k == num {
        for i in 0..k {
            a[i as usize] = i;
        }

        return;
    }

    if k > num - k {
        build_subset(num, a, random);
        invert(num, a);
    } else {
        build_subset(num, a, random);
    }
}

fn build_subset(n: i32, sub: &mut Vec<i32>, random: &mut ThreadRng) {
    let k = sub.len() as i32;
    check_subset(n, k);

    if sub.len() as i32 == n {
        for i in 0..k {
            sub[i as usize] = i;
        }
        return;
    }

    for i in 0..k {
        sub[i as usize] = i * n / k;
    }

    let mut l;
    let mut ix;
    for _ in 0..k {
        loop {
            ix = random.gen_range(1..n);
            l = (ix * k - 1) / n;
            if sub[l as usize] < ix {
                break;
            }
        }

        sub[l as usize] = sub[l as usize] + 1;
    }

    let mut m = 0;
    let mut ip = 0;
    let mut is_ = k;
    for i in 0..k {
        m = sub[i as usize];
        sub[i as usize] = 0;

        if m != i * n / k {
            ip = ip + 1;
            sub[ip as usize - 1] = m;
        }
    }

    let ihi = ip;
    for i in 1..=ihi {
        ip = ihi + 1 - i;
        l = 1 + (sub[ip as usize - 1] * k - 1) / n;
        let ids = sub[ip as usize - 1] - (l - 1) * n / k;
        sub[ip as usize - 1] = 0;
        sub[is_ as usize - 1] = l;
        is_ = is_ - ids;
    }

    let mut ir = 0;
    let mut m0 = 0;
    for ll in 1..=k {
        l = k + 1 - ll;

        if sub[l as usize - 1] != 0 {
            ir = l;
            m0 = 1 + (sub[l as usize - 1] - 1) * n / k;
            m = sub[l as usize - 1] * n / k - m0 + 1;
        }

        ix = random.gen_range(m0..m0 + m - 1);

        let mut i = l + 1;
        while i <= ir && ix >= sub[i as usize - 1] {
            ix = ix + 1;
            sub[i as usize - 2] = sub[i as usize - 1];
            i = i + 1;
        }

        sub[i as usize - 2] = ix;
        m -= 1;
    }
}

fn invert(n: i32, a: &mut Vec<i32>) {
    let k = a.len() as i32;

    let mut v = n - 1;
    let mut j = n - k - 1;
    let mut vi;

    let mut ac = vec![0; k as usize];
    ac.copy_from_slice(&a);

    for i in (0..k).rev() {
        loop {
            vi = index_of(&ac, j, v);
            if vi != -1 {
                break;
            }
            v -= 1;
            j = vi;
        }

        a[i as usize] = v;
        v -= 1;
    }
}

fn index_of(a: &Vec<i32>, start: i32, value: i32) -> i32 {
    for i in (0..=start).rev() {
        if a[i as usize] < value {
            return -1;
        } else if a[i as usize] == value {
            return i;
        }
    }

    -1
}

fn check_subset(n: i32, k: i32) {
    if k <= 0 {
        panic!("Subset size smaller or equal zero: {}", k);
    }
    if n < k {
        panic!("n smaller than k: {} < {}.", n, k);
    }
}
