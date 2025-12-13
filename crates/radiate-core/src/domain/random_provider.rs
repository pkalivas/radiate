use rand::distr::{Distribution, StandardUniform, uniform::SampleUniform};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, RngCore, SeedableRng};
use std::cell::RefCell;
use std::ops::Range;
use std::sync::{Arc, LazyLock, Mutex};

static GLOBAL_RNG: LazyLock<Arc<Mutex<SmallRng>>> =
    LazyLock::new(|| Arc::new(Mutex::new(SmallRng::from_os_rng())));

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

/// Seeds the thread-local random number generator with the given seed.
pub fn set_seed(seed: u64) {
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
pub fn gaussian(mean: f64, std_dev: f64) -> f64 {
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
    pub fn bool(&mut self, prob: f32) -> bool {
        self.0.random_bool(prob as f64)
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
    pub fn gaussian(&mut self, mean: f64, std_dev: f64) -> f64 {
        let u1: f64 = self.0.random();
        let u2: f64 = self.0.random();
        let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        mean + std_dev * z0
    }

    #[inline]
    pub fn shuffled_indices(&mut self, range: Range<usize>) -> Vec<usize> {
        let mut indexes = range.collect::<Vec<usize>>();
        indexes.shuffle(&mut self.0);
        indexes
    }

    #[inline]
    pub fn sample_indices(&mut self, range: Range<usize>, sample_size: usize) -> Vec<usize> {
        let mut indexes = range.collect::<Vec<usize>>();
        indexes.shuffle(&mut self.0);
        indexes.truncate(sample_size);
        indexes
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
}
