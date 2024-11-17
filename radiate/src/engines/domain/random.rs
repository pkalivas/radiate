use rand::{distributions::Standard, prelude::Distribution, rngs::StdRng, Rng, SeedableRng};
use std::sync::{Arc, Mutex, OnceLock};


static INSTANCE: OnceLock<RandomRegistry> = OnceLock::new();

/// A registry to manage randomness globally in an application.
pub struct RandomRegistry {
    rng: Arc<Mutex<StdRng>>,
}

impl RandomRegistry {
    /// Returns the global instance of the registry.
    pub fn global() -> &'static RandomRegistry {
        INSTANCE.get_or_init(|| RandomRegistry {
            rng: Arc::new(Mutex::new(StdRng::from_entropy())),
        })
    }

    /// Sets a new seed for the global RNG.
    pub fn set_seed(&self, seed: u64) {
        let mut rng = self.rng.lock().unwrap();
        *rng = StdRng::seed_from_u64(seed);
    }

    /// Generates a random number using the global RNG.
    pub fn random<T>(&self) -> T
    where
        T: rand::distributions::uniform::SampleUniform,
        Standard: Distribution<T>
    {
        let mut rng = self.rng.lock().unwrap();
        rng.gen()
    }

    /// Executes a function with a temporary seeded RNG.
    pub fn with_seed<F, T>(&self, seed: u64, func: F) -> T
    where
        F: FnOnce(&mut StdRng) -> T,
    {
        let mut temp_rng = StdRng::seed_from_u64(seed);
        func(&mut temp_rng)
    }
}