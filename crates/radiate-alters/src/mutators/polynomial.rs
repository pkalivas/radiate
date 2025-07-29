use radiate_core::{Chromosome, FloatGene, Gene, Mutate, random_provider};

#[derive(Debug, Clone, PartialEq)]
pub struct PolynomialMutator {
    rate: f32,
    eta: f32,
}

impl PolynomialMutator {
    pub fn new(rate: f32, eta: f32) -> Self {
        if !(0.0..=1.0).contains(&rate) {
            panic!("Rate must be between 0 and 1");
        }
        if eta <= 0.0 {
            panic!("Eta must be positive");
        }

        PolynomialMutator { rate, eta }
    }

    fn polynomial_mutation(&self, value: f32, min: f32, max: f32) -> f32 {
        let u = random_provider::random::<f32>();

        // Calculate normalized distances
        let delta1 = (value - min) / (max - min);
        let delta2 = (max - value) / (max - min);

        let mutq: f32;

        if u <= 0.5 {
            // Left side of the polynomial
            let term1 = 2.0 * u;
            let term2 = (1.0 - 2.0 * u) * (1.0 - delta1).powf(self.eta + 1.0);
            mutq = (term1 + term2).powf(1.0 / (self.eta + 1.0));
        } else {
            // Right side of the polynomial
            let term1 = 2.0 * (1.0 - u);
            let term2 = 2.0 * (u - 0.5) * (1.0 - delta2).powf(self.eta + 1.0);
            mutq = 1.0 - (term1 + term2).powf(1.0 / (self.eta + 1.0));
        }

        // Ensure the result is within bounds
        min + mutq * (max - min)
    }
}

impl<C: Chromosome<Gene = FloatGene>> Mutate<C> for PolynomialMutator {
    fn rate(&self) -> f32 {
        self.rate
    }

    #[inline]
    fn mutate_gene(&self, gene: &C::Gene) -> C::Gene {
        let min = gene.bounds().start;
        let max = gene.bounds().end;
        let value = *gene.allele();

        let new_value = self.polynomial_mutation(value, min, max);

        // Ensure the new value is within bounds
        let clamped_value = new_value.clamp(min, max);

        gene.with_allele(&clamped_value)
    }
}
