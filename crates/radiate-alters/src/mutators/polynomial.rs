use radiate_core::{BoundedGene, Chromosome, FloatGene, Gene, Mutate, random_provider};
use std::sync::{Mutex, RwLock};

// Use it when:
// 	- You’re evolving floating-point representations (like real-valued neural nets, control parameters, orbital mechanics).
// 	- You want bounded and unbiased mutation behavior.
// 	- You care about the distribution of the mutations. Unlike Gaussian, polynomial gives more control over tail behavior.

// It’s especially powerful in:
// 	- Multi-objective algorithms (like NSGA-II)
// 	- High-precision tuning problems
// 	- Bounded domains where classic Gaussian mutation may overshoot

// The eta parameter in polynomial mutation controls the shape of the mutation distribution.
// In other words, how local or global your mutations are:
// 	- eta is the distribution index (usually denoted as η_m in literature like Deb’s NSGA-II paper).
// 	- It determines the exploration vs. exploitation trade-off:
// 	- Low eta (e.g. 1–5): leads to bigger mutations, promoting exploration.
// 	- High eta (e.g. 20–100): leads to smaller, fine-grained mutations, good for local search.
#[derive(Debug)]
pub struct PolynomialMutator {
    rate: f32,
    eta: RwLock<f32>,
    initial_eta: Option<Mutex<f32>>,
    decay: Option<Mutex<f32>>,
}

impl PolynomialMutator {
    pub fn new(rate: f32, eta: f32) -> Self {
        if !(0.0..=1.0).contains(&rate) {
            panic!("Rate must be between 0 and 1");
        }
        if eta <= 0.0 {
            panic!("Eta must be positive");
        }

        PolynomialMutator {
            rate,
            eta: RwLock::new(eta),
            initial_eta: Some(Mutex::new(eta)),
            decay: None,
        }
    }

    pub fn with_decay(mut self, decay: f32) -> Self {
        if decay <= 0.0 {
            panic!("Decay must be positive");
        }

        self.decay = Some(Mutex::new(decay));
        self
    }

    fn polynomial_mutation(&self, value: f32, min: f32, max: f32, eta: f32) -> f32 {
        let u = random_provider::random::<f32>();

        if (max - min).abs() < f32::EPSILON {
            return value;
        }

        let delta1 = (value - min) / (max - min);
        let delta2 = (max - value) / (max - min);

        let mutq = if u <= 0.5 {
            // Left side of the polynomial
            let term1 = 2.0 * u;
            let term2 = (1.0 - 2.0 * u) * (1.0 - delta1).powf(eta + 1.0);
            (term1 + term2).powf(1.0 / (eta + 1.0))
        } else {
            // Right side of the polynomial
            let term1 = 2.0 * (1.0 - u);
            let term2 = 2.0 * (u - 0.5) * (1.0 - delta2).powf(eta + 1.0);
            1.0 - (term1 + term2).powf(1.0 / (eta + 1.0))
        };

        min + mutq * (max - min)
    }
}

impl<C: Chromosome<Gene = FloatGene>> Mutate<C> for PolynomialMutator {
    fn rate(&self) -> f32 {
        self.rate
    }

    fn update(&self, generation: usize) {
        if let (Some(init), Some(decay)) = (&self.initial_eta, &self.decay) {
            let init = init.lock().unwrap();
            let decay = decay.lock().unwrap();
            let mut eta = self.eta.write().unwrap();
            *eta = *init / (1.0 + *decay * generation as f32);
        }
    }

    #[inline]
    fn mutate_gene(&self, gene: &C::Gene) -> C::Gene {
        // TODO: Should these be from the bounds?
        let min = *gene.min();
        let max = *gene.max();
        let value = *gene.allele();
        let eta = *self.eta.read().unwrap();

        let new_value = self.polynomial_mutation(value, min, max, eta);

        let clamped_value = new_value.clamp(min, max);

        gene.with_allele(&clamped_value)
    }
}
