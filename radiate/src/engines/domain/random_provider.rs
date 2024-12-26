use rand::distributions::Standard;
use rand::distributions::{uniform::SampleUniform, Distribution};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::Rng;
use rand::SeedableRng;
use std::sync::Mutex;

thread_local! {
    static RNG: Mutex<StdRng> = Mutex::new(StdRng::from_entropy());
}

/// Seeds the thread-local random number generator with the given seed.
pub fn seed_rng(seed: u64) {
    RNG.with(|rng| {
        *rng.lock().unwrap() = StdRng::seed_from_u64(seed);
    });
}

/// Generates a random number of type T.
///
/// For floating point types, the number will be in the range [0, 1).
/// For integer types, the number will be in the range [0, MAX).
pub fn random<T>() -> T
where
    T: SampleUniform,
    Standard: Distribution<T>,
{
    RNG.with(|rng| rng.lock().unwrap().gen())
}

/// Generates a random number of type T in the given range.
pub fn gen_range<T>(range: std::ops::Range<T>) -> T
where
    T: SampleUniform + PartialOrd,
    Standard: Distribution<T>,
{
    RNG.with(|rng| rng.lock().unwrap().gen_range(range))
}

/// Chooses a random item from the given slice.
pub fn choose<T>(items: &[T]) -> &T {
    RNG.with(|rng| {
        let index = rng.lock().unwrap().gen_range(0..items.len());
        &items[index]
    })
}

/// Generates a random number from a Gaussian distribution with the given mean and standard deviation.
/// The Box-Muller transform is used to generate the random number.
pub fn gaussian(mean: f64, std_dev: f64) -> f64 {
    RNG.with(|rng| {
        let mut rng = rng.lock().unwrap();

        // Generate two independent random numbers in the range (0, 1]
        let u1: f64 = rng.gen(); // Uniform random number
        let u2: f64 = rng.gen();

        // Apply the Box-Muller transform
        let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();

        // Scale and shift to match the desired mean and standard deviation
        mean + std_dev * z0
    })
}

/// Shuffles the given slice in place.
pub fn shuffle<T>(items: &mut [T]) {
    RNG.with(|rng| items.shuffle(&mut *rng.lock().unwrap()));
}

/// Generates a vector of indexes from 0 to n-1 in random order.
pub fn indexes(n: usize) -> Vec<usize> {
    let mut indexes: Vec<usize> = (0..n).collect();
    shuffle(&mut indexes);
    indexes
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_random() {
        for _ in 0..100 {
            let value: f64 = random();
            assert!(value >= 0.0 && value < 1.0);
        }
    }

    #[test]
    fn test_gen_range() {
        for _ in 0..100 {
            let value: f64 = gen_range(0.0..100.0);
            assert!(value >= 0.0 && value < 100.0);
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
    fn test_seeded_gaussian() {
        let mean = 0.0;
        let std_dev = 1.0;
        let mut sum = 0.0;
        for _ in 0..100 {
            sum += gaussian(mean, std_dev);
        }

        let average = sum / 100.0;
        assert!((average - mean).abs() < 0.1);
    }

    #[test]
    fn test_shuffle() {
        let mut items = vec![1, 2, 3, 4, 5];
        shuffle(&mut items);
        assert_ne!(items, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_indexes() {
        let indexes = indexes(10);
        assert_eq!(indexes.len(), 10);
        assert_ne!(indexes, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }
}
