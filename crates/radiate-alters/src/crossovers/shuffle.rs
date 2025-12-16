use radiate_core::{AlterResult, Chromosome, Crossover, Rate, Valid, random_provider};

pub struct ShuffleCrossover {
    rate: Rate,
}

impl ShuffleCrossover {
    pub fn new(rate: impl Into<Rate>) -> Self {
        let rate = rate.into();
        if !rate.is_valid() {
            panic!("Rate {rate:?} is not valid. Must be between 0.0 and 1.0",);
        }
        ShuffleCrossover { rate }
    }
}

impl<C: Chromosome + Clone> Crossover<C> for ShuffleCrossover {
    fn rate(&self) -> Rate {
        self.rate.clone()
    }

    #[inline]
    fn cross_chromosomes(&self, chrom_one: &mut C, chrom_two: &mut C, rate: f32) -> AlterResult {
        let length = std::cmp::min(chrom_one.len(), chrom_two.len());
        if length < 2 {
            return AlterResult::empty();
        }

        let mut cross_count = 0;

        random_provider::with_rng(|rand| {
            let mut indices = (0..length).collect::<Vec<usize>>();
            rand.shuffle(&mut indices);

            let temp_chrom_one = chrom_one.genes_mut();
            let temp_chrom_two = chrom_two.genes_mut();

            for (i, &index) in indices.iter().enumerate() {
                if i % 2 == 0 {
                    if !rand.bool(rate) {
                        continue;
                    }

                    std::mem::swap(&mut temp_chrom_one[index], &mut temp_chrom_two[index]);
                    cross_count += 1;
                }
            }
        });

        AlterResult::from(cross_count)
    }
}

#[cfg(test)]
mod tests {

    use super::ShuffleCrossover;
    use radiate_core::{Chromosome, Crossover, Gene, IntChromosome};

    #[test]
    fn test_shuffle_crossover() {
        let mut chrom_one = IntChromosome::from(vec![1, 2, 3, 4, 5]);
        let mut chrom_two = IntChromosome::from(vec![6, 7, 8, 9, 10]);

        let crossover = ShuffleCrossover::new(1.0);
        let result = crossover.cross_chromosomes(&mut chrom_one, &mut chrom_two, 1.0);

        let one_alleles = chrom_one.iter().map(|g| *g.allele()).collect::<Vec<i32>>();
        let two_alleles = chrom_two.iter().map(|g| *g.allele()).collect::<Vec<i32>>();

        assert_ne!(one_alleles, vec![1, 2, 3, 4, 5]);
        assert_ne!(two_alleles, vec![6, 7, 8, 9, 10]);
        assert!(result.0 > 0);
    }

    #[test]
    fn test_shuffle_crossover_no_effect() {
        let mut chrom_one = IntChromosome::from(vec![1, 2, 3, 4, 5]);
        let mut chrom_two = IntChromosome::from(vec![6, 7, 8, 9, 10]);

        let crossover = ShuffleCrossover::new(0.0);
        let result = crossover.cross_chromosomes(&mut chrom_one, &mut chrom_two, 0.0);

        let one_alleles = chrom_one.iter().map(|g| *g.allele()).collect::<Vec<i32>>();
        let two_alleles = chrom_two.iter().map(|g| *g.allele()).collect::<Vec<i32>>();

        assert_eq!(one_alleles, vec![1, 2, 3, 4, 5]);
        assert_eq!(two_alleles, vec![6, 7, 8, 9, 10]);
        assert_eq!(result.0, 0);
    }
}
