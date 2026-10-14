use radiate_core::{
    AlterContext, BoundedGene, Chromosome, Crossover, Expr, Gene, Phenotype, RateSet,
    random_provider,
};
use radiate_utils::Float;

/// A target plus three distinct donors.
const MIN_POPULATION_SIZE: usize = 4;

/// Differential evolution as a crossover (DE/rand/1/bin) for float chromosomes.
///
/// Each individual becomes a *target* with probability `rate`. For each target, three other
/// distinct individuals `a`, `b` and `c` are drawn, and every gene of the target is replaced
/// with probability `crossover_rate` by the donor value:
///
/// ```text
///   v = a + weight · (b − c)
/// ```
///
/// One randomly chosen gene per chromosome always takes `v`, so every target changes. Values
/// are clamped to each gene's bounds.
///
/// All trials are built from the population as it was before this step, so a target never
/// draws a donor that was already changed in the same step, matching standard DE.
///
/// Textbook DE keeps a trial only if it scores better than its target. An alterer can't
/// evaluate, so here the trial always replaces the target and the engine's survivor
/// selection does the filtering, the usual way DE is used as an operator inside a GA.
#[derive(Debug, Clone, PartialEq)]
pub struct DifferentialCrossover {
    rate: Expr,
    weight: f32,
    crossover_rate: f32,
}

impl DifferentialCrossover {
    /// `rate`: chance each individual becomes a target. `weight` (often called F): how far
    /// along the difference `b − c` the donor moves, usually 0.4 to 0.9. `crossover_rate`
    /// (often called CR): chance each gene takes the donor's value, usually 0.1 to 0.9.
    ///
    /// In general, a higher crossover rate means that more genes of the trial will come from the donor
    /// and vice versa for a lower crossover rate. IE:
    ///
    /// - crossover_rate: 0.9 if the variables interact, 0.1 if they're independent.
    /// - weight (F): 0.5 is the standard starting point. Raise it toward 0.8 if the population converges too early.
    pub fn new(rate: impl Into<Expr>, weight: f32, crossover_rate: f32) -> Self {
        if !(weight > 0.0 && weight <= 2.0) {
            panic!("Weight must be in (0, 2], got {weight}");
        }

        if !(0.0..=1.0).contains(&crossover_rate) {
            panic!("Crossover rate must be between 0 and 1, got {crossover_rate}");
        }

        DifferentialCrossover {
            rate: rate.into(),
            weight,
            crossover_rate,
        }
    }
}

impl<A, G, C> Crossover<C> for DifferentialCrossover
where
    A: Float,
    G: Gene<Allele = A> + BoundedGene,
    C: Chromosome<Gene = G>,
{
    fn rates(&self) -> RateSet {
        RateSet::new(self.rate.clone())
    }

    fn crossover(&self, population: &mut [Phenotype<C>], ctx: &mut AlterContext) -> usize {
        let size = population.len();
        if size < MIN_POPULATION_SIZE {
            return 0;
        }

        let weight = A::from(self.weight).unwrap();

        // Build every trial first: (target, chromosome, gene, value).
        let mut changes = Vec::new();
        for target in 0..size {
            if !random_provider::bool(ctx.rate()) {
                continue;
            }

            let [a, b, c] = donors(target, size);
            let geno_idxs = [target, a, b, c].map(|i| population[i].genotype());

            let chromosomes = geno_idxs.iter().map(|g| g.len()).min().unwrap_or(0);
            for chrom_idx in 0..chromosomes {
                let genes = geno_idxs
                    .iter()
                    .map(|g| g[chrom_idx].len())
                    .min()
                    .unwrap_or(0);

                if genes == 0 {
                    continue;
                }

                // `allele(k)` reads this gene from genotypes[k]: 1 = a, 2 = b, 3 = c.
                let forced_idx = random_provider::range(0..genes);
                for gene_idx in 0..genes {
                    if gene_idx != forced_idx && !random_provider::bool(self.crossover_rate) {
                        continue;
                    }

                    let allele =
                        |k: usize| *geno_idxs[k][chrom_idx].get(gene_idx).unwrap().allele();
                    let value = allele(1) + weight * (allele(2) - allele(3));
                    changes.push((target, chrom_idx, gene_idx, value));
                }
            }
        }

        // Then write them all.
        let mut last_target = None;
        for &(target, chromosome, gene, value) in &changes {
            let phenotype = &mut population[target];
            if let Some(gene) = phenotype.genotype_mut()[chromosome].get_mut(gene) {
                gene.set_allele(value);
            }

            if last_target != Some(target) {
                phenotype.invalidate(ctx.generation());
                last_target = Some(target);
            }
        }

        changes.len()
    }
}

/// Three distinct indices in `0..size`, none equal to `target`.
fn donors(target: usize, size: usize) -> [usize; 3] {
    let mut picked = random_provider::sample_indices(0..size, 4)
        .into_iter()
        .filter(|&i| i != target);

    [
        picked.next().unwrap(),
        picked.next().unwrap(),
        picked.next().unwrap(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use radiate_core::{FloatChromosome, FloatGene, alter::AlterUpdates};

    fn phenotype(values: &[f32], bound: f32) -> Phenotype<FloatChromosome<f32>> {
        let genes = values
            .iter()
            .map(|&v| FloatGene::new(v, -bound..bound, -bound..bound))
            .collect::<Vec<_>>();
        Phenotype::from((vec![FloatChromosome::new(genes)], 0))
    }

    fn alleles(phenotype: &Phenotype<FloatChromosome<f32>>) -> Vec<f32> {
        phenotype.genotype()[0]
            .iter()
            .map(|g| *g.allele())
            .collect()
    }

    fn cross(
        crossover: &DifferentialCrossover,
        population: &mut [Phenotype<FloatChromosome<f32>>],
        rate: f32,
    ) -> usize {
        let mut updates = AlterUpdates::default();
        let mut ctx = AlterContext::new(&mut updates, 0, rate, &[]);
        crossover.crossover(population, &mut ctx)
    }

    #[test]
    fn test_trials_follow_the_formula_from_the_original_population() {
        random_provider::scoped_seed(1, || {
            let original = [
                vec![1.0, 2.0, 3.0],
                vec![-4.0, 0.5, 9.0],
                vec![7.0, -1.0, 2.0],
                vec![0.0, 6.0, -5.0],
                vec![3.0, 3.0, 3.0],
            ];
            let mut population = original
                .iter()
                .map(|v| phenotype(v, 100.0))
                .collect::<Vec<_>>();

            let crossover = DifferentialCrossover::new(1.0, 0.5, 1.0);
            assert_eq!(cross(&crossover, &mut population, 1.0), 5 * 3);

            // Every target became a + 0.5·(b − c) for some ordering of three *other* original
            // individuals, read before any target was written.
            for (target, phenotype) in population.iter().enumerate() {
                let got = alleles(phenotype);
                let others = (0..original.len())
                    .filter(|&i| i != target)
                    .collect::<Vec<_>>();

                let matches = others.iter().any(|&a| {
                    others.iter().any(|&b| {
                        others.iter().any(|&c| {
                            a != b
                                && b != c
                                && a != c
                                && (0..3).all(|g| {
                                    let want =
                                        original[a][g] + 0.5 * (original[b][g] - original[c][g]);
                                    (got[g] - want).abs() < 1e-6
                                })
                        })
                    })
                });

                assert!(matches, "target {target} got {got:?}");
            }
        });
    }

    #[test]
    fn test_zero_crossover_rate_still_changes_one_gene_per_target() {
        random_provider::scoped_seed(2, || {
            let mut population = (0..6)
                .map(|i| phenotype(&[i as f32, -(i as f32), 2.0 * i as f32, 1.0], 100.0))
                .collect::<Vec<_>>();

            let crossover = DifferentialCrossover::new(1.0, 0.8, 0.0);
            assert_eq!(cross(&crossover, &mut population, 1.0), 6);
        });
    }

    #[test]
    fn test_values_are_clamped_to_gene_bounds() {
        random_provider::scoped_seed(3, || {
            let mut population = [[0.9, -0.9], [-0.9, 0.9], [0.9, 0.9], [-0.9, -0.9]]
                .iter()
                .map(|v| phenotype(v, 1.0))
                .collect::<Vec<_>>();

            let crossover = DifferentialCrossover::new(1.0, 2.0, 1.0);
            cross(&crossover, &mut population, 1.0);

            for phenotype in &population {
                assert!(alleles(phenotype).iter().all(|v| (-1.0..=1.0).contains(v)));
            }
        });
    }

    #[test]
    fn test_small_population_and_zero_rate_are_untouched() {
        let crossover = DifferentialCrossover::new(1.0, 0.5, 0.9);

        let mut small = [[1.0], [2.0], [3.0]]
            .iter()
            .map(|v| phenotype(v, 10.0))
            .collect::<Vec<_>>();
        assert_eq!(cross(&crossover, &mut small, 1.0), 0);

        let mut population = (0..8)
            .map(|i| phenotype(&[i as f32], 10.0))
            .collect::<Vec<_>>();
        assert_eq!(cross(&crossover, &mut population, 0.0), 0);
        assert!(
            population
                .iter()
                .enumerate()
                .all(|(i, p)| alleles(p) == vec![i as f32])
        );
    }

    #[test]
    #[should_panic(expected = "Weight must be in (0, 2]")]
    fn test_rejects_zero_weight() {
        DifferentialCrossover::new(1.0, 0.0, 0.5);
    }
}
