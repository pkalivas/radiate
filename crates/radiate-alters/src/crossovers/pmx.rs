use radiate_core::{
    AlterResult, Chromosome, Crossover, PermutationChromosome, Rate, SubsetMode, Valid,
    math::indexes,
};

pub struct PMXCrossover {
    rate: Rate,
}

impl PMXCrossover {
    pub fn new(rate: impl Into<Rate>) -> Self {
        let rate = rate.into();
        if !rate.is_valid() {
            panic!("Rate {rate:?} is not valid. Must be between 0.0 and 1.0",);
        }

        PMXCrossover { rate }
    }
}

impl<A: PartialEq + Clone> Crossover<PermutationChromosome<A>> for PMXCrossover {
    fn rate(&self) -> Rate {
        self.rate.clone()
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut PermutationChromosome<A>,
        chrom_two: &mut PermutationChromosome<A>,
        _: f32,
    ) -> AlterResult {
        let length = std::cmp::min(chrom_one.genes().len(), chrom_two.genes().len());
        if length < 2 {
            return AlterResult::empty();
        }

        let subset = indexes::subset(chrom_one.genes.len(), 2, SubsetMode::StratifiedCorrect);

        // start will always be less than end due to StratifiedCorrect
        let start = subset[0];
        let end = subset[1];

        let mut offspring_one = chrom_one.genes.clone();
        let mut offspring_two = chrom_two.genes.clone();

        offspring_one[start..(end + 1)].clone_from_slice(&chrom_two.genes[start..(end + 1)]);
        offspring_two[start..(end + 1)].clone_from_slice(&chrom_one.genes[start..(end + 1)]);

        for i in 0..length {
            if i < start || i > end {
                let mut gene_one = chrom_one.get(i);
                let mut gene_two = chrom_two.get(i);

                while offspring_one[start..=end].contains(gene_one) {
                    let index = chrom_two.genes.iter().position(|g| g == gene_one).unwrap();
                    gene_one = chrom_one.get(index);
                }

                while offspring_two[start..=end].contains(gene_two) {
                    let index = chrom_one.genes.iter().position(|g| g == gene_two).unwrap();
                    gene_two = chrom_two.get(index);
                }

                offspring_one[i] = gene_one.clone();
                offspring_two[i] = gene_two.clone();
            }
        }

        chrom_one.genes = offspring_one;
        chrom_two.genes = offspring_two;

        AlterResult::from(2)
    }
}
