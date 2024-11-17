use rand::distributions::Standard;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::sync::{Arc, Mutex};
use std::sync::OnceLock;

/// A registry to manage randomness globally in an application.
pub struct RandomRegistry {
    rng: Arc<Mutex<StdRng>>,
}

impl RandomRegistry {
    /// Returns the global instance of the registry.
    pub fn global() -> &'static Self {
        static INSTANCE: OnceLock<RandomRegistry> = OnceLock::new();
        INSTANCE.get_or_init(|| RandomRegistry {
            rng: Arc::new(Mutex::new(StdRng::from_entropy())),
        })
    }

    /// Sets a new seed for the global RNG.
    pub fn set_seed(seed: u64) {
        let global = RandomRegistry::global();
        let mut rng = global.rng.lock().unwrap();
        *rng = StdRng::seed_from_u64(seed);
    }

    /// Generates a random number using the global RNG.
    pub fn random<T>() -> T
    where
        T: rand::distributions::uniform::SampleUniform,
        Standard: rand::distributions::Distribution<T>,
    {
        let global = RandomRegistry::global();
        let mut rng = global.rng.lock().unwrap();
        rng.gen()
    }

    pub fn gen_range<T>(range: std::ops::Range<T>) -> T
    where
        T: rand::distributions::uniform::SampleUniform + PartialOrd,
        Standard: rand::distributions::Distribution<T>,
    {
        let global = RandomRegistry::global();
        let mut rng = global.rng.lock().unwrap();
        rng.gen_range(range)
    }

    /// Executes a function with a temporary seeded RNG.
    pub fn with_seed<F, T>(seed: u64, func: F) -> T
    where
        F: FnOnce(&mut StdRng) -> T,
    {
        let mut temp_rng = StdRng::seed_from_u64(seed);
        func(&mut temp_rng)
    }

    pub fn choose<T>(items: &[T]) -> &T {
        let global = RandomRegistry::global();
        let mut rng = global.rng.lock().unwrap();
        let index = rng.gen_range(0..items.len());
        &items[index]
    }
}