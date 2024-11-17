use rand::distributions::Standard;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::sync::OnceLock;
use std::sync::{Arc, Mutex};

pub struct RandomRegistry {
    rng: Arc<Mutex<StdRng>>,
}

impl RandomRegistry {
    pub fn global() -> &'static Self {
        static INSTANCE: OnceLock<RandomRegistry> = OnceLock::new();
        INSTANCE.get_or_init(|| RandomRegistry {
            rng: Arc::new(Mutex::new(StdRng::from_entropy())),
        })
    }

    pub fn set_seed(seed: u64) {
        let global = RandomRegistry::global();
        let mut rng = global.rng.lock().unwrap();
        *rng = StdRng::seed_from_u64(seed);
    }

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

// #[cfg(test)]
// mod tests {

//     use super::*;

//     #[test]
//     fn test_random_registry() {
//         RandomRegistry::set_seed(42);
//         let random_int: i32 = RandomRegistry::random();
//         assert!(random_int == 572990626);
//     }

//     #[test]
//     fn random_registry_choose_produces_same_result_with_seed() {
//         RandomRegistry::set_seed(42);
//         let items = vec![1, 2, 3, 4, 5];
//         let choice = RandomRegistry::choose(&items);
//         assert_eq!(choice, &4);
//     }

//     #[test]
//     fn random_registry_gen_range_produces_same_value_with_seed() {
//         RandomRegistry::set_seed(42);
//         let value = RandomRegistry::gen_range(0..10);
//         assert_eq!(value, 1);
//     }
// }
