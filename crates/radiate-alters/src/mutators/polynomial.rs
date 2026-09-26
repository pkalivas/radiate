use radiate_core::{
    AlterContext, BoundedGene, Chromosome, Expr, FloatGene, Gene, Mutate, RateSet, random_provider,
};
use radiate_utils::{Float, Primitive};

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
#[derive(Debug, Clone)]
pub struct PolynomialMutator {
    rate: Expr,
    eta: f32,
}

impl PolynomialMutator {
    pub fn new(rate: impl Into<Expr>, eta: f32) -> Self {
        PolynomialMutator {
            rate: rate.into(),
            eta,
        }
    }

    /// Deb's bounded polynomial mutation (as used in NSGA-II / pymoo). The perturbation
    /// `delta_q` lies in `[-delta1, delta2]`, so the result is `value` shifted by at most
    /// the distance to either bound - the larger `eta`, the closer `delta_q` is to 0.
    fn polynomial_mutation(&self, value: f64, min: f64, max: f64, eta: f64) -> f64 {
        let u = random_provider::random::<f64>();

        if (max - min).abs() < f64::EPSILON {
            return value;
        }

        let delta1 = (value - min) / (max - min);
        let delta2 = (max - value) / (max - min);
        let mut_pow = 1.0 / (eta + 1.0);

        let delta_q = if u <= 0.5 {
            // Left side of the polynomial - moves the value towards `min`
            let term1 = 2.0 * u;
            let term2 = (1.0 - 2.0 * u) * (1.0 - delta1).powf(eta + 1.0);
            (term1 + term2).powf(mut_pow) - 1.0
        } else {
            // Right side of the polynomial - moves the value towards `max`
            let term1 = 2.0 * (1.0 - u);
            let term2 = 2.0 * (u - 0.5) * (1.0 - delta2).powf(eta + 1.0);
            1.0 - (term1 + term2).powf(mut_pow)
        };

        value + delta_q * (max - min)
    }
}

impl<F, C> Mutate<C> for PolynomialMutator
where
    F: Float + Primitive,
    C: Chromosome<Gene = FloatGene<F>>,
{
    fn rates(&self) -> RateSet {
        RateSet::new(self.rate.clone())
    }

    #[inline]
    fn mutate_chromosome(&mut self, chromosome: &mut C, ctx: &mut AlterContext) -> usize {
        let mut count = 0;
        for gene in chromosome.iter_mut() {
            if random_provider::bool(ctx.rate()) {
                let (lower, upper) = gene.bound_range();
                let min = lower.extract::<f64>().unwrap();
                let max = upper.extract::<f64>().unwrap();
                let value = gene.allele().extract::<f64>().unwrap();
                let eta = self.eta as f64;

                // `new_value` here is clamped safely within the bounds when the allele is set.
                let new_value = self.polynomial_mutation(value, min, max, eta);
                gene.set_allele(F::from(new_value).unwrap());

                count += 1;
            }
        }

        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLES: usize = 20_000;

    fn sample(mutator: &PolynomialMutator, value: f64, min: f64, max: f64) -> Vec<f64> {
        (0..SAMPLES)
            .map(|_| mutator.polynomial_mutation(value, min, max, mutator.eta as f64))
            .collect()
    }

    fn mean_abs_shift(samples: &[f64], value: f64) -> f64 {
        samples.iter().map(|y| (y - value).abs()).sum::<f64>() / samples.len() as f64
    }

    #[test]
    fn polynomial_mutation_is_a_local_perturbation() {
        random_provider::seed(42);
        let mutator = PolynomialMutator::new(1.0, 20.0);

        // With eta = 20 Deb's operator moves a value by ~0.03-0.05 of the range on average.
        // A formula that is anchored at `min` rather than `value` lands ~0.45 away.
        for value in [0.05, 0.3, 0.5, 0.7, 0.95] {
            let samples = sample(&mutator, value, 0.0, 1.0);
            let shift = mean_abs_shift(&samples, value);
            assert!(shift < 0.1, "value {value}: mean |y - x| = {shift}");
        }
    }

    #[test]
    fn polynomial_mutation_stays_in_bounds() {
        random_provider::seed(7);
        let mutator = PolynomialMutator::new(1.0, 1.0);

        for value in [-10.0, -9.99, -3.0, 0.0, 4.2, 9.99, 10.0] {
            for y in sample(&mutator, value, -10.0, 10.0) {
                assert!(
                    (-10.0..=10.0).contains(&y),
                    "value {value} mutated out of bounds: {y}"
                );
            }
        }
    }

    #[test]
    fn polynomial_mutation_is_centered_on_value() {
        random_provider::seed(11);
        let mutator = PolynomialMutator::new(1.0, 20.0);

        // Midpoint of the range: the distribution is symmetric, so roughly half the
        // samples should fall on either side and the mean should stay at the value.
        let samples = sample(&mutator, 5.0, 0.0, 10.0);
        let below = samples.iter().filter(|&&y| y < 5.0).count() as f64 / SAMPLES as f64;
        let mean = samples.iter().sum::<f64>() / SAMPLES as f64;

        assert!((below - 0.5).abs() < 0.03, "fraction below value = {below}");
        assert!((mean - 5.0).abs() < 0.05, "mean = {mean}");
    }

    #[test]
    fn polynomial_mutation_larger_eta_is_more_local() {
        random_provider::seed(3);

        let shifts = [2.0, 20.0, 100.0].map(|eta| {
            let mutator = PolynomialMutator::new(1.0, eta);
            mean_abs_shift(&sample(&mutator, 0.4, 0.0, 1.0), 0.4)
        });

        assert!(
            shifts[0] > shifts[1] && shifts[1] > shifts[2],
            "expected mean shift to shrink as eta grows, got {shifts:?}"
        );
    }
}
