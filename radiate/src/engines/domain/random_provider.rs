use rand::distributions::Standard;
use rand::distributions::{uniform::SampleUniform, Distribution};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::sync::OnceLock;
use std::sync::{Arc, Mutex};

static INSTANCE: OnceLock<RandomProvider> = OnceLock::new();

pub struct RandomProvider {
    rng: Arc<Mutex<StdRng>>,
}

impl RandomProvider {
    pub fn global() -> &'static Self {
        INSTANCE.get_or_init(|| RandomProvider {
            rng: Arc::new(Mutex::new(StdRng::from_entropy())),
        })
    }

    pub fn set_seed(seed: u64) {
        let global = RandomProvider::global();
        let mut rng = global.rng.lock().unwrap();
        *rng = StdRng::seed_from_u64(seed);
    }

    pub fn random<T>() -> T
    where
        T: SampleUniform,
        Standard: Distribution<T>,
    {
        let global = RandomProvider::global();
        let mut rng = global.rng.lock().unwrap();
        rng.gen()
    }

    pub fn gen_range<T>(range: std::ops::Range<T>) -> T
    where
        T: SampleUniform + PartialOrd,
        Standard: Distribution<T>,
    {
        let global = RandomProvider::global();
        let mut rng = global.rng.lock().unwrap();
        rng.gen_range(range)
    }

    pub fn choose<T>(items: &[T]) -> &T {
        let global = RandomProvider::global();
        let mut rng = global.rng.lock().unwrap();
        let index = rng.gen_range(0..items.len());
        &items[index]
    }
}
