use crate::domain::env_vars;
use radiate_utils::Float;
use rand::distr::{Distribution, StandardUniform, uniform::SampleUniform};
use rand::rngs::SmallRng;
use rand::rngs::SysRng;
use rand::seq::SliceRandom;
use rand::{Rng, RngExt, SeedableRng};
use std::cell::RefCell;
use std::ops::Range;
use std::sync::{Arc, LazyLock, Mutex};

/// Probability above which [bernoulli_indices] switches from geometric gap sampling to
/// one draw per index. See its docs for the benchmark behind this value.
const BERNOULLI_THRESHOLD: f64 = 0.45;
const TWO_POW_64: f64 = 18_446_744_073_709_551_616.0;
const TWO_POW_NEG_53: f64 = 1.0 / 9_007_199_254_740_992.0;

static GLOBAL_RNG: LazyLock<Arc<Mutex<SmallRng>>> = LazyLock::new(|| {
    let maybe_seed = env_vars::seed();
    Arc::new(Mutex::new(match maybe_seed {
        Some(seed) => SmallRng::seed_from_u64(seed),
        None => SmallRng::try_from_rng(&mut SysRng).unwrap(),
    }))
});

thread_local! {
    static TLS_RNG: RefCell<SmallRng> = RefCell::new({
        let mut global = GLOBAL_RNG.lock().unwrap();
        SmallRng::seed_from_u64(global.next_u64())
    });
}

pub fn with_rng<R>(f: impl FnOnce(&mut RdRand<'_>) -> R) -> R {
    TLS_RNG.with(|cell| {
        let mut rng = cell.borrow_mut();
        f(&mut RdRand::new(&mut rng))
    })
}

/// Calls `f(i)` for each index in `range`, where each index is selected independently
/// with probability `p` - the same result as `if bool(p) { f(i) }` for every index,
/// but usually much cheaper. `p <= 0` or `NaN` selects nothing and `p >= 1` selects
/// every index. Indices are passed to `f` in ascending order.
///
/// Two strategies are used, chosen by `p`:
///
/// - **Geometric (low `p`)**: the number of unselected indices before the next selected
///   one follows a geometric distribution, so it's sampled directly and the loop jumps
///   straight to the next selected index. This costs one random draw per *selected*
///   index (plus one) instead of one per index.
/// - **Per-index (high `p`)**: one Bernoulli draw per index. When most indices are
///   selected anyway, this is cheaper than the logarithm the geometric strategy needs
///   per selection.
///
/// Benchmark over 1000 indices, in ns per call:
///
/// | `p`    | Geometric | Per-index |
/// |-------:|----------:|----------:|
/// | 0.0002 |        19 |      1245 |
/// | 0.01   |       109 |      1257 |
/// | 0.1    |       699 |      1696 |
/// | 0.4    |      2691 |      3435 |
/// | 0.5    |      3357 |      3782 |
/// | 0.7    |      4749 |      2692 |
///
/// The switch happens at `p = 0.45`, on the conservative side of where the two cross.
///
/// The RNG is only borrowed for each draw, not while `f` runs, so `f` is free to use
/// `random_provider` itself (e.g. `gene.new_instance()`). If `f` doesn't need
/// randomness, [RdRand::bernoulli_indices] avoids re-borrowing the RNG for each draw.
pub fn bernoulli_indices<F: Float>(p: F, range: Range<usize>, f: impl FnMut(usize)) {
    bernoulli_select(p, range, random::<u64>, f);
}

/// Seeds the thread-local random number generator with the given seed.
pub fn seed(seed: u64) {
    let mut global = GLOBAL_RNG.lock().unwrap();
    *global = SmallRng::seed_from_u64(seed);
}

/// Temporarily sets the seed of the thread-local random number generator to the given seed
/// for the duration of the closure `f`. After `f` completes, the original state of the RNG is restored.
pub fn scoped_seed<R>(seed: u64, f: impl FnOnce() -> R) -> R {
    TLS_RNG.with(|cell| {
        let original_seed = {
            let mut rng = cell.borrow_mut();
            let original = rng.clone();
            *rng = SmallRng::seed_from_u64(seed);
            original
        };

        let result = f();

        let mut rng = cell.borrow_mut();
        *rng = original_seed;

        result
    })
}

///
/// For floating point types, the number will be in the range [0, 1).
/// For integer types, the number will be in the range [0, MAX).
#[inline(always)]
pub fn random<T>() -> T
where
    T: SampleUniform,
    StandardUniform: Distribution<T>,
{
    with_rng(|rng| rng.random())
}

/// Generates a random boolean with the given probability of being true.
#[inline(always)]
pub fn bool(prob: f32) -> bool {
    with_rng(|rng| rng.bool(prob))
}

/// Generates a random number of type T in the given range.
pub fn range<T>(range: Range<T>) -> T
where
    T: SampleUniform + PartialOrd,
{
    with_rng(|rng| rng.range(range))
}

/// Chooses a random item from the given slice.
pub fn choose<T>(items: &[T]) -> &T {
    with_rng(|rng| rng.choose(items))
}

pub fn choose_mut<T>(items: &mut [T]) -> &mut T {
    with_rng(|rng| rng.choose_mut(items))
}

/// Generates a random number from a Gaussian distribution with the given mean and standard deviation.
/// The Box-Muller transform is used to generate the random number.
pub fn gaussian<T>(mean: T, std_dev: T) -> T
where
    T: SampleUniform + Float,
{
    with_rng(|rng| rng.gaussian(mean, std_dev))
}

/// Shuffles the given slice in place.
pub fn shuffle<T>(items: &mut [T]) {
    with_rng(|rng| rng.shuffle(items));
}

/// Generates a vector of indexes from 0 to n-1 in random order.
pub fn shuffled_indices(range: Range<usize>) -> Vec<usize> {
    with_rng(|rng| rng.shuffled_indices(range))
}

/// Returns `sample_size` distinct indices from `range`, in random order. If `sample_size`
/// is larger than the range, every index in the range is returned (shuffled).
///
/// Only the sampled indices are generated - the cost is roughly `O(sample_size)`,
/// not `O(range.len())` - so this stays cheap when sampling a few points from a long range.
pub fn sample_indices(range: Range<usize>, sample_size: usize) -> Vec<usize> {
    with_rng(|rng| rng.sample_indices(range, sample_size))
}

/// Returns a vector of indexes from the given range, each included with the given probability.
pub fn cond_indices(range: Range<usize>, prob: f32) -> Vec<usize> {
    with_rng(|rng| rng.cond_indices(range, prob))
}

pub struct RdRand<'a>(&'a mut SmallRng);

impl<'a> RdRand<'a> {
    pub fn new(rng: &'a mut SmallRng) -> Self {
        RdRand(rng)
    }

    #[inline]
    pub fn random<T>(&mut self) -> T
    where
        T: SampleUniform,
        StandardUniform: Distribution<T>,
    {
        self.0.random()
    }

    #[inline]
    pub fn range<T>(&mut self, range: Range<T>) -> T
    where
        T: SampleUniform + PartialOrd,
    {
        self.0.random_range(range)
    }

    #[inline]
    pub fn bool<F: Float>(&mut self, prob: F) -> bool {
        prob.extract::<f64>()
            .map(|val| self.0.random_bool(val))
            .unwrap_or_default()
    }

    #[inline]
    pub fn choose<'b, T>(&mut self, items: &'b [T]) -> &'b T {
        let index = self.0.random_range(0..items.len());
        &items[index]
    }

    #[inline]
    pub fn choose_mut<'b, T>(&mut self, items: &'b mut [T]) -> &'b mut T {
        let index = self.0.random_range(0..items.len());
        &mut items[index]
    }

    #[inline]
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        items.shuffle(&mut self.0);
    }

    #[inline]
    pub fn gaussian<T>(&mut self, mean: T, std_dev: T) -> T
    where
        T: Float,
    {
        let u1: f64 = self.0.random();
        let u2: f64 = self.0.random();
        let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        mean + std_dev * T::from(z0).unwrap()
    }

    #[inline]
    pub fn shuffled_indices(&mut self, range: Range<usize>) -> Vec<usize> {
        let mut indexes = range.collect::<Vec<usize>>();
        indexes.shuffle(&mut self.0);
        indexes
    }

    #[inline]
    pub fn sample_indices(&mut self, range: Range<usize>, sample_size: usize) -> Vec<usize> {
        let len = range.len();
        rand::seq::index::sample(self.0, len, sample_size.min(len))
            .into_iter()
            .map(|i| range.start + i)
            .collect()
    }

    #[inline]
    pub fn cond_indices(&mut self, range: Range<usize>, prob: f32) -> Vec<usize> {
        if prob >= 1.0 {
            return range.collect();
        }

        if prob <= 0.0 {
            return Vec::new();
        }

        range.filter(|_| self.0.random::<f32>() < prob).collect()
    }

    /// Same as [bernoulli_indices], but draws from this RNG. The RNG stays borrowed while
    /// `f` runs, so `f` must not call `random_provider` - use the free function if it does.
    #[inline]
    pub fn bernoulli_indices<F: Float>(&mut self, p: F, range: Range<usize>, f: impl FnMut(usize)) {
        bernoulli_select(p, range, || self.0.next_u64(), f);
    }
}

/// Shared implementation of [bernoulli_indices] and [RdRand::bernoulli_indices].
/// `next_u64` returns uniformly random bits.
#[inline]
fn bernoulli_select<F: Float>(
    p: F,
    range: Range<usize>,
    mut next_u64: impl FnMut() -> u64,
    mut f: impl FnMut(usize),
) {
    let Some(p) = p.extract::<f64>() else {
        return;
    };

    if p.is_nan() || p <= 0.0 || range.is_empty() {
        return;
    }

    if p >= 1.0 {
        range.for_each(f);
        return;
    }

    if p > BERNOULLI_THRESHOLD {
        // Same integer comparison `rand`'s `random_bool` uses - noticeably cheaper per index
        // than converting each draw to an f64. `p` is in (0, 1) here, so this can't overflow.
        let threshold = (p * TWO_POW_64) as u64;
        for i in range {
            if next_u64() < threshold {
                f(i);
            }
        }

        return;
    }

    // The number of unselected indices before the next selected one is geometric:
    // P(gap >= k) = (1 - p)^k, so gap = floor(ln(1 - u) / ln(1 - p)).
    // `ln_1p` keeps ln(1 - p) accurate (and non-zero) for tiny p.
    let ln_q = (-p).ln_1p();
    let (mut i, end) = (range.start, range.end);
    loop {
        // Uniform in [0, 1) from the top 53 bits, the same conversion `rand` uses for f64.
        let u = (next_u64() >> 11) as f64 * TWO_POW_NEG_53;
        let gap = ((1.0 - u).ln() / ln_q).floor();

        // Next selection is past the end (also exits on a NaN or infinite gap)
        if gap.is_nan() || gap >= (end - i) as f64 {
            break;
        }

        i += gap as usize;
        f(i);
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random() {
        for _ in 0..100 {
            let value: f64 = random();
            assert!((0.0..1.0).contains(&value));
        }
    }

    #[test]
    fn test_gen_range() {
        for _ in 0..100 {
            let value: f64 = range(0.0..100.0);
            assert!((0.0..100.0).contains(&value));
        }
    }

    #[test]
    fn test_choose() {
        for _ in 0..100 {
            let items = vec![1, 2, 3, 4, 5];
            let value = choose(&items);
            assert!(items.contains(value));
        }
    }

    #[test]
    fn test_shuffle() {
        let mut items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        shuffle(&mut items);
        assert_ne!(items, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    #[test]
    fn test_indexes() {
        let indexes = shuffled_indices(0..10);
        assert_eq!(indexes.len(), 10);
        assert_ne!(indexes, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn sample_indices_returns_distinct_indices_within_range() {
        for (range, size) in [(0..10, 3), (5..25, 4), (100..1_100, 2), (3..4, 1)] {
            for _ in 0..500 {
                let mut picks = sample_indices(range.clone(), size);
                assert_eq!(picks.len(), size);
                assert!(picks.iter().all(|i| range.contains(i)), "{picks:?}");

                picks.sort_unstable();
                picks.dedup();
                assert_eq!(picks.len(), size, "duplicate indices");
            }
        }
    }

    #[test]
    fn sample_indices_edge_cases() {
        assert!(sample_indices(0..0, 3).is_empty());
        assert!(sample_indices(0..10, 0).is_empty());

        // Asking for more than the range holds returns the whole range, shuffled.
        let mut all = sample_indices(4..9, 20);
        all.sort_unstable();
        assert_eq!(all, vec![4, 5, 6, 7, 8]);
    }

    #[test]
    fn sample_indices_picks_every_index_uniformly() {
        const TRIALS: usize = 100_000;
        let mut counts = [0usize; 10];
        for _ in 0..TRIALS {
            for i in sample_indices(0..10, 3) {
                counts[i] += 1;
            }
        }

        // Each of the 10 indices is in a 3-sample with probability 0.3.
        let expected = TRIALS as f64 * 0.3;
        for (i, &count) in counts.iter().enumerate() {
            let deviation = (count as f64 - expected).abs() / expected;
            assert!(
                deviation < 0.02,
                "index {i}: {count} vs ~{expected:.0}, {counts:?}"
            );
        }
    }

    fn selected(p: f32, range: Range<usize>) -> Vec<usize> {
        let mut out = Vec::new();
        bernoulli_indices(p, range, |i| out.push(i));
        out
    }

    #[test]
    fn bernoulli_indices_selects_every_position_with_probability_p() {
        // Covers both strategies: geometric gaps (p <= threshold) and per-index draws.
        const TRIALS: usize = 200_000;
        const LEN: usize = 10;

        for p in [0.01f32, 0.3, 0.45, 0.7] {
            let mut counts = [0usize; LEN];
            for _ in 0..TRIALS {
                bernoulli_indices(p, 0..LEN, |i| counts[i] += 1);
            }

            let p = p as f64;
            let tolerance = 5.0 * (p * (1.0 - p) / TRIALS as f64).sqrt();
            for (i, &count) in counts.iter().enumerate() {
                let rate = count as f64 / TRIALS as f64;
                assert!(
                    (rate - p).abs() < tolerance,
                    "p = {p}: index {i} selected at rate {rate}, counts = {counts:?}"
                );
            }
        }
    }

    #[test]
    fn bernoulli_indices_yields_sorted_unique_indices_within_range() {
        for p in [0.05f32, 0.3, 0.7] {
            for _ in 0..1_000 {
                let picks = selected(p, 5..25);
                assert!(picks.windows(2).all(|w| w[0] < w[1]), "{picks:?}");
                assert!(picks.iter().all(|i| (5..25).contains(i)), "{picks:?}");
            }
        }
    }

    #[test]
    fn bernoulli_indices_reaches_single_and_last_index() {
        const TRIALS: usize = 20_000;
        let mut single = 0;
        let mut last = 0;
        for _ in 0..TRIALS {
            bernoulli_indices(0.3, 0..1, |_| single += 1);
            bernoulli_indices(0.3, 0..10, |i| last += (i == 9) as usize);
        }

        // Expected ~6000 each; zero would mean the final index is never selected.
        assert!((5_000..7_000).contains(&single), "single = {single}");
        assert!((5_000..7_000).contains(&last), "last = {last}");
    }

    #[test]
    fn bernoulli_indices_edge_cases() {
        assert!(selected(0.5, 0..0).is_empty());
        assert!(selected(0.5, 7..7).is_empty());

        for p in [0.0f32, -0.1, -1.0, f32::NAN, f32::NEG_INFINITY] {
            assert!(selected(p, 0..100).is_empty(), "p = {p}");
        }

        // Rates too small to represent as 1 - p in f64 must still select (almost) nothing.
        for p in [1e-17f32, 1e-30, f32::MIN_POSITIVE] {
            assert!(selected(p, 0..1_000).is_empty(), "p = {p}");
        }

        for p in [1.0f32, 1.5, f32::INFINITY] {
            assert_eq!(selected(p, 3..8), vec![3, 4, 5, 6, 7], "p = {p}");
        }
    }

    #[test]
    fn bernoulli_indices_callback_can_use_random_provider() {
        let mut values = Vec::new();
        bernoulli_indices(0.2, 0..1_000, |_| values.push(random::<f32>()));
        bernoulli_indices(0.8, 0..1_000, |_| values.push(range(0.0..1.0)));
        assert!(!values.is_empty());
    }

    #[test]
    fn rd_rand_bernoulli_indices_matches_free_function() {
        let mut picks = Vec::new();
        with_rng(|rng| rng.bernoulli_indices(1.0f32, 0..4, |i| picks.push(i)));
        assert_eq!(picks, vec![0, 1, 2, 3]);

        let mut count = 0;
        with_rng(|rng| {
            for _ in 0..10_000 {
                rng.bernoulli_indices(0.1f32, 0..10, |_| count += 1);
            }
        });

        // Expected 10,000 * 10 * 0.1 = 10,000 selections.
        assert!((9_400..10_600).contains(&count), "count = {count}");
    }
}
