use rand::distributions::{uniform::SampleUniform, Distribution};
use rand::distributions::{Standard, Uniform};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::sync::Mutex;

thread_local! {
    static RNG: Mutex<StdRng> = Mutex::new(StdRng::from_entropy());
}

pub fn seed_rng(seed: u64) {
    RNG.with(|rng| {
        *rng.lock().unwrap() = StdRng::seed_from_u64(seed);
    });
}

pub fn random<T>() -> T
where
    T: SampleUniform,
    Standard: Distribution<T>,
{
    RNG.with(|rng| rng.lock().unwrap().gen())
}

pub fn gen_range<T>(range: std::ops::Range<T>) -> T
where
    T: SampleUniform + PartialOrd,
    Standard: Distribution<T>,
{
    RNG.with(|rng| rng.lock().unwrap().gen_range(range))
}

pub fn choose<T>(items: &[T]) -> &T {
    RNG.with(|rng| {
        let index = rng.lock().unwrap().gen_range(0..items.len());
        &items[index]
    })
}

pub fn gauss(mean: f64, std_dev: f64) -> f64 {
    RNG.with(|rng| {
        let x = rng.lock().unwrap().sample(Uniform::new(0.0, 1.0));
        let y = rng.lock().unwrap().sample(Uniform::new(0.0, 1.0));
        let z = (mean + std_dev * (2.0 * x - 1.0) * (2.0 * std_dev * y).sqrt()).abs();

        z
    })
}
