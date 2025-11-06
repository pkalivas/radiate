use radiate_core::{AlterResult, Chromosome, Crossover, FloatGene, Gene, random_provider};

/// The [BlendCrossover] is a crossover operator that blends [FloatGene] alleles from two parent chromosomes to create offspring.
/// The blending is controlled by the `alpha` parameter, which determines the extent of blending between the two alleles.
/// The formula used for blending is:
///
/// ```text
/// new_allele_one = allele_one - (alpha * (allele_two - allele_one))
/// new_allele_two = allele_two - (alpha * (allele_one - allele_two))
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct BlendCrossover {
    rate: f32,
    alpha: f32,
}

impl BlendCrossover {
    /// Create a new instance of the [BlendCrossover] with the given rate and alpha.
    /// The rate must be between 0.0 and 1.0, and the alpha must be between 0.0 and 1.0.
    pub fn new(rate: f32, alpha: f32) -> Self {
        if !(0.0..=1.0).contains(&rate) {
            panic!("Rate must be between 0 and 1");
        }

        if !(0.0..=1.0).contains(&alpha) {
            panic!("Alpha must be between 0 and 1");
        }

        BlendCrossover { rate, alpha }
    }
}

impl<C> Crossover<C> for BlendCrossover
where
    C: Chromosome<Gene = FloatGene>,
{
    fn rate(&self) -> f32 {
        self.rate
    }

    #[inline]
    fn cross_chromosomes(&self, chrom_one: &mut C, chrom_two: &mut C, rate: f32) -> AlterResult {
        let mut cross_count = 0;

        for i in 0..std::cmp::min(chrom_one.len(), chrom_two.len()) {
            if random_provider::bool(rate) {
                let gene_one = chrom_one.get(i);
                let gene_two = chrom_two.get(i);

                let allele_one: f32 = gene_one.allele().clone();
                let allele_two: f32 = gene_two.allele().clone();

                let new_allele_one = allele_one - (self.alpha * (allele_two - allele_one));
                let new_allele_two = allele_two - (self.alpha * (allele_one - allele_two));

                chrom_one.set(i, gene_one.with_allele(&new_allele_one));
                chrom_two.set(i, gene_two.with_allele(&new_allele_two));

                cross_count += 1;
            }
        }

        cross_count.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use radiate_core::{FloatChromosome, FloatGene};

    #[test]
    fn test_cross_chromosomes_basic() {
        let crossover = BlendCrossover::new(1.0, 0.5);

        let genes1 = vec![
            FloatGene::new(1.0, 0.0..10.0, 0.0..10.0),
            FloatGene::new(2.0, 0.0..10.0, 0.0..10.0),
            FloatGene::new(3.0, 0.0..10.0, 0.0..10.0),
        ];
        let genes2 = vec![
            FloatGene::new(4.0, 0.0..10.0, 0.0..10.0),
            FloatGene::new(5.0, 0.0..10.0, 0.0..10.0),
            FloatGene::new(6.0, 0.0..10.0, 0.0..10.0),
        ];

        let mut chrom_one = FloatChromosome::new(genes1);
        let mut chrom_two = FloatChromosome::new(genes2);

        let original_one: Vec<f32> = chrom_one.iter().map(|g| *g.allele()).collect();
        let original_two: Vec<f32> = chrom_two.iter().map(|g| *g.allele()).collect();

        let result = crossover.cross_chromosomes(&mut chrom_one, &mut chrom_two, 1.0);

        assert_eq!(result.count(), 3);

        // Check that values have been blended according to the formula
        // new_allele_one = allele_one - (alpha * (allele_two - allele_one))
        // new_allele_two = allele_two - (alpha * (allele_one - allele_two))
        let alpha = 0.5;

        for i in 0..chrom_one.len() {
            let expected_one = original_one[i] - (alpha * (original_two[i] - original_one[i]));
            let expected_two = original_two[i] - (alpha * (original_one[i] - original_two[i]));

            assert!((chrom_one.get(i).allele() - expected_one).abs() < 1e-6);
            assert!((chrom_two.get(i).allele() - expected_two).abs() < 1e-6);
        }
    }

    #[test]
    fn test_cross_chromosomes_zero_rate() {
        let crossover = BlendCrossover::new(0.0, 0.5);

        let genes1 = vec![
            FloatGene::new(1.0, 0.0..10.0, 0.0..10.0),
            FloatGene::new(2.0, 0.0..10.0, 0.0..10.0),
        ];
        let genes2 = vec![
            FloatGene::new(4.0, 0.0..10.0, 0.0..10.0),
            FloatGene::new(5.0, 0.0..10.0, 0.0..10.0),
        ];

        let mut chrom_one = FloatChromosome::new(genes1);
        let mut chrom_two = FloatChromosome::new(genes2);

        let original_one: Vec<f32> = chrom_one.iter().map(|g| *g.allele()).collect();
        let original_two: Vec<f32> = chrom_two.iter().map(|g| *g.allele()).collect();

        let result = crossover.cross_chromosomes(&mut chrom_one, &mut chrom_two, 0.0);

        assert_eq!(result.count(), 0);

        // Values should remain unchanged
        for i in 0..chrom_one.len() {
            assert_eq!(*chrom_one.get(i).allele(), original_one[i]);
            assert_eq!(*chrom_two.get(i).allele(), original_two[i]);
        }
    }

    #[test]
    fn test_cross_chromosomes_different_lengths() {
        let crossover = BlendCrossover::new(1.0, 0.3);

        let genes1 = vec![
            FloatGene::new(1.0, 0.0..10.0, 0.0..10.0),
            FloatGene::new(2.0, 0.0..10.0, 0.0..10.0),
            FloatGene::new(3.0, 0.0..10.0, 0.0..10.0),
        ];
        let genes2 = vec![
            FloatGene::new(4.0, 0.0..10.0, 0.0..10.0),
            FloatGene::new(5.0, 0.0..10.0, 0.0..10.0),
        ];

        let mut chrom_one = FloatChromosome::new(genes1);
        let mut chrom_two = FloatChromosome::new(genes2);

        let result = crossover.cross_chromosomes(&mut chrom_one, &mut chrom_two, 1.0);

        assert_eq!(result.count(), 2);

        let alpha = 0.3;
        let expected_one_0 = 1.0 - (alpha * (4.0 - 1.0));
        let expected_two_0 = 4.0 - (alpha * (1.0 - 4.0));
        let expected_one_1 = 2.0 - (alpha * (5.0 - 2.0));
        let expected_two_1 = 5.0 - (alpha * (2.0 - 5.0));

        assert!((chrom_one.get(0).allele() - expected_one_0).abs() < 1e-6);
        assert!((chrom_two.get(0).allele() - expected_two_0).abs() < 1e-6);
        assert!((chrom_one.get(1).allele() - expected_one_1).abs() < 1e-6);
        assert!((chrom_two.get(1).allele() - expected_two_1).abs() < 1e-6);

        assert_eq!(*chrom_one.get(2).allele(), 3.0);
    }

    #[test]
    fn test_cross_chromosomes_alpha_zero() {
        let crossover = BlendCrossover::new(1.0, 0.0);

        let genes1 = vec![
            FloatGene::new(1.0, 0.0..10.0, 0.0..10.0),
            FloatGene::new(2.0, 0.0..10.0, 0.0..10.0),
        ];
        let genes2 = vec![
            FloatGene::new(4.0, 0.0..10.0, 0.0..10.0),
            FloatGene::new(5.0, 0.0..10.0, 0.0..10.0),
        ];

        let mut chrom_one = FloatChromosome::new(genes1);
        let mut chrom_two = FloatChromosome::new(genes2);

        let original_one: Vec<f32> = chrom_one.iter().map(|g| *g.allele()).collect();
        let original_two: Vec<f32> = chrom_two.iter().map(|g| *g.allele()).collect();

        let result = crossover.cross_chromosomes(&mut chrom_one, &mut chrom_two, 1.0);

        assert_eq!(result.count(), 2);

        // With alpha = 0, values should remain unchanged
        for i in 0..chrom_one.len() {
            assert_eq!(*chrom_one.get(i).allele(), original_one[i]);
            assert_eq!(*chrom_two.get(i).allele(), original_two[i]);
        }
    }

    #[test]
    fn test_cross_chromosomes_alpha_one() {
        let crossover = BlendCrossover::new(1.0, 1.0);

        let genes1 = vec![
            FloatGene::new(1.0, 0.0..10.0, 0.0..10.0),
            FloatGene::new(2.0, 0.0..10.0, 0.0..10.0),
        ];
        let genes2 = vec![
            FloatGene::new(4.0, 0.0..10.0, 0.0..10.0),
            FloatGene::new(5.0, 0.0..10.0, 0.0..10.0),
        ];

        let mut chrom_one = FloatChromosome::new(genes1);
        let mut chrom_two = FloatChromosome::new(genes2);

        let result = crossover.cross_chromosomes(&mut chrom_one, &mut chrom_two, 1.0);

        assert_eq!(result.count(), 2);

        // With alpha = 1, values should be swapped
        assert_eq!(*chrom_one.get(0).allele(), -2.0);
        assert_eq!(*chrom_two.get(0).allele(), 7.0);
        assert_eq!(*chrom_one.get(1).allele(), -1.0);
        assert_eq!(*chrom_two.get(1).allele(), 8.0);
    }

    #[test]
    fn test_cross_chromosomes_identical_parents() {
        let crossover = BlendCrossover::new(1.0, 0.5);

        let genes = vec![
            FloatGene::new(1.0, 0.0..10.0, 0.0..10.0),
            FloatGene::new(2.0, 0.0..10.0, 0.0..10.0),
        ];

        let mut chrom_one = FloatChromosome::new(genes.clone());
        let mut chrom_two = FloatChromosome::new(genes);

        let result = crossover.cross_chromosomes(&mut chrom_one, &mut chrom_two, 1.0);

        assert_eq!(result.count(), 2);

        // With identical parents, values should remain the same
        for i in 0..chrom_one.len() {
            assert_eq!(*chrom_one.get(i).allele(), *chrom_two.get(i).allele());
        }
    }

    #[test]
    fn test_cross_chromosomes_property_based() {
        let crossover = BlendCrossover::new(1.0, 0.5);

        for _ in 0..50 {
            let genes1: Vec<FloatGene> = (0..5)
                .map(|_| {
                    FloatGene::new(
                        random_provider::random::<f32>() * 10.0,
                        0.0..10.0,
                        0.0..10.0,
                    )
                })
                .collect();
            let genes2: Vec<FloatGene> = (0..5)
                .map(|_| {
                    FloatGene::new(
                        random_provider::random::<f32>() * 10.0,
                        0.0..10.0,
                        0.0..10.0,
                    )
                })
                .collect();

            let mut chrom_one = FloatChromosome::new(genes1);
            let mut chrom_two = FloatChromosome::new(genes2);

            let original_one: Vec<f32> = chrom_one.iter().map(|g| *g.allele()).collect();
            let original_two: Vec<f32> = chrom_two.iter().map(|g| *g.allele()).collect();

            let result = crossover.cross_chromosomes(&mut chrom_one, &mut chrom_two, 1.0);

            assert_eq!(result.count(), 5);

            let alpha = 0.5;
            for i in 0..chrom_one.len() {
                let expected_one = original_one[i] - (alpha * (original_two[i] - original_one[i]));
                let expected_two = original_two[i] - (alpha * (original_one[i] - original_two[i]));

                assert!((chrom_one.get(i).allele() - expected_one).abs() < 1e-6);
                assert!((chrom_two.get(i).allele() - expected_two).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn test_cross_chromosomes_edge_cases() {
        let crossover = BlendCrossover::new(1.0, 0.5);

        // Test with single gene chromosomes
        let genes1 = vec![FloatGene::new(1.0, 0.0..10.0, 0.0..10.0)];
        let genes2 = vec![FloatGene::new(4.0, 0.0..10.0, 0.0..10.0)];

        let mut chrom_one = FloatChromosome::new(genes1);
        let mut chrom_two = FloatChromosome::new(genes2);

        let result = crossover.cross_chromosomes(&mut chrom_one, &mut chrom_two, 1.0);

        assert_eq!(result.count(), 1);

        // Test with empty chromosomes (should not panic)
        let mut empty_one = FloatChromosome::new(vec![]);
        let mut empty_two = FloatChromosome::new(vec![]);

        let result = crossover.cross_chromosomes(&mut empty_one, &mut empty_two, 1.0);
        assert_eq!(result.count(), 0);
    }

    #[test]
    fn test_blend_formula_verification() {
        let crossover = BlendCrossover::new(1.0, 0.3);
        let alpha = 0.3;

        // Test specific values to verify the blending formula
        let genes1 = vec![FloatGene::new(2.0, 0.0..10.0, 0.0..10.0)];
        let genes2 = vec![FloatGene::new(8.0, 0.0..10.0, 0.0..10.0)];

        let mut chrom_one = FloatChromosome::new(genes1);
        let mut chrom_two = FloatChromosome::new(genes2);

        crossover.cross_chromosomes(&mut chrom_one, &mut chrom_two, 1.0);

        // Manual calculation: allele_one = 2.0, allele_two = 8.0, alpha = 0.3
        // new_allele_one = 2.0 - (0.3 * (8.0 - 2.0)) = 2.0 - (0.3 * 6.0) = 2.0 - 1.8 = 0.2
        // new_allele_two = 8.0 - (0.3 * (2.0 - 8.0)) = 8.0 - (0.3 * -6.0) = 8.0 + 1.8 = 9.8
        let expected_one = 2.0 - (alpha * (8.0 - 2.0));
        let expected_two = 8.0 - (alpha * (2.0 - 8.0));

        assert!((chrom_one.get(0).allele() - expected_one).abs() < 1e-6);
        assert!((chrom_two.get(0).allele() - expected_two).abs() < 1e-6);
    }
}
