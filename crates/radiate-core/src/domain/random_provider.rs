use rand::distr::{Distribution, StandardUniform, uniform::SampleUniform};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, RngCore, SeedableRng};
use std::cell::RefCell;
use std::ops::Range;
use std::sync::{Arc, LazyLock, Mutex};

static GLOBAL_RNG: LazyLock<Arc<Mutex<StdRng>>> =
    LazyLock::new(|| Arc::new(Mutex::new(StdRng::from_os_rng())));

thread_local! {
    static TLS_RNG: RefCell<StdRng> = RefCell::new({
        let mut global = GLOBAL_RNG.lock().unwrap();
        StdRng::seed_from_u64(global.next_u64())
    });
}

pub fn with_rng<R>(f: impl FnOnce(&mut StdRng) -> R) -> R {
    TLS_RNG.with(|cell| {
        let mut rng = cell.borrow_mut();
        f(&mut *rng)
    })
}

/// Seeds the thread-local random number generator with the given seed.
pub fn set_seed(seed: u64) {
    let mut global = GLOBAL_RNG.lock().unwrap();
    *global = StdRng::seed_from_u64(seed);
}

pub fn reseed_current_thread() {
    let mut global = GLOBAL_RNG.lock().unwrap();
    TLS_RNG.with(|cell| {
        *cell.borrow_mut() = StdRng::seed_from_u64(global.next_u64());
    });
}

pub fn set_seed_and_reseed_current(seed: u64) {
    {
        let mut global = GLOBAL_RNG.lock().unwrap();
        *global = StdRng::seed_from_u64(seed);
    }

    reseed_current_thread();
}

/// Generates a random number of type T.
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
    with_rng(|rng| rng.random_bool(prob as f64))
}

/// Generates a random number of type T in the given range.
pub fn range<T>(range: Range<T>) -> T
where
    T: SampleUniform + PartialOrd,
{
    with_rng(|rng| rng.random_range(range))
}

/// Chooses a random item from the given slice.
pub fn choose<T>(items: &[T]) -> &T {
    with_rng(|rng| {
        let index = rng.random_range(0..items.len());
        &items[index]
    })
}

pub fn choose_mut<T>(items: &mut [T]) -> &mut T {
    with_rng(|rng| {
        let index = rng.random_range(0..items.len());
        &mut items[index]
    })
}

/// Generates a random number from a Gaussian distribution with the given mean and standard deviation.
/// The Box-Muller transform is used to generate the random number.
pub fn gaussian(mean: f64, std_dev: f64) -> f64 {
    with_rng(|rng| {
        let u1: f64 = rng.random();
        let u2: f64 = rng.random();

        let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();

        mean + std_dev * z0
    })
}

/// Shuffles the given slice in place.
pub fn shuffle<T>(items: &mut [T]) {
    with_rng(|rng| items.shuffle(rng));
}

/// Generates a vector of indexes from 0 to n-1 in random order.
pub fn shuffled_indices(range: Range<usize>) -> Vec<usize> {
    with_rng(|rng| {
        let mut indexes = range.collect::<Vec<usize>>();
        indexes.shuffle(rng);
        indexes
    })
}

pub fn cond_indices(range: Range<usize>, prob: f32) -> Vec<usize> {
    with_rng(|rng| {
        if prob >= 1.0 {
            return range.collect();
        }

        if prob <= 0.0 {
            return Vec::new();
        }

        range.filter(|_| rng.random::<f32>() < prob).collect()
    })
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

// struct RandomProvider {
//     rng: Arc<Mutex<StdRng>>,
// }

// impl RandomProvider {
//     /// Returns the global instance of the registry.
//     pub(self) fn global() -> &'static RandomProvider {
//         static INSTANCE: OnceLock<RandomProvider> = OnceLock::new();

//         INSTANCE.get_or_init(|| RandomProvider {
//             rng: Arc::new(Mutex::new(StdRng::from_os_rng())),
//         })
//     }

//     pub(self) fn get_rng() -> StdRng {
//         let instance = Self::global();
//         let rng = instance.rng.lock().unwrap();
//         rng.clone()
//     }

//     /// Sets a new seed for the global RNG.
//     pub(self) fn set_seed(seed: u64) {
//         let instance = Self::global();
//         let mut rng = instance.rng.lock().unwrap();
//         *rng = StdRng::seed_from_u64(seed);
//     }

//     /// Generates a random number using the global RNG.
//     pub(self) fn random<T>() -> T
//     where
//         T: SampleUniform,
//         StandardUniform: Distribution<T>,
//     {
//         let instance = Self::global();
//         let mut rng = instance.rng.lock().unwrap();
//         rng.random()
//     }

//     pub(self) fn range<T>(range: Range<T>) -> T
//     where
//         T: SampleUniform + PartialOrd,
//     {
//         let instance = Self::global();
//         let mut rng = instance.rng.lock().unwrap();

//         rng.random_range(range)
//     }

//     pub(self) fn bool(prob: f64) -> bool {
//         let instance = Self::global();
//         let mut rng = instance.rng.lock().unwrap();
//         rng.random_bool(prob)
//     }
// }
