use radiate_core::{
    AlterContext, AlterResult, Chromosome, Crossover, Expr, Expr, ExprSet, PermutationChromosome,
    SubsetMode, math::indexes,
};

const PMX_CROSSOVER_RATE: &str = "crossover.pmx.rate";

pub struct PMXCrossover {
    rate: Expr,
}

impl PMXCrossover {
    pub fn new(rate: impl Into<Expr>) -> Self {
        PMXCrossover { rate: rate.into().alias(PMX_CROSSOVER_RATE) }
    }
}

impl<A: PartialEq + Clone> Crossover<PermutationChromosome<A>> for PMXCrossover {
    fn rates(&self) -> ExprSet {
        ExprSet::from(self.rate.clone())
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut PermutationChromosome<A>,
        chrom_two: &mut PermutationChromosome<A>,
        _: &mut AlterContext,
    ) -> AlterResult {
        let length = std::cmp::min(chrom_one.as_slice().len(), chrom_two.as_slice().len());
        if length < 2 {
            return AlterResult::empty();
        }

        let mut subset = vec![0; 2];
        indexes::subset(
            chrom_one.genes.len(),
            2,
            &mut subset,
            SubsetMode::StratifiedCorrect,
        );

        // start will always be less than end due to StratifiedCorrect
        let start = subset[0];
        let end = subset[1];

        let mut offspring_one = chrom_one.genes.clone();
        let mut offspring_two = chrom_two.genes.clone();

        offspring_one[start..(end + 1)].clone_from_slice(&chrom_two.genes[start..(end + 1)]);
        offspring_two[start..(end + 1)].clone_from_slice(&chrom_one.genes[start..(end + 1)]);

        for i in 0..length {
            if i < start || i > end {
                let mut gene_one = chrom_one.get(i).expect("Gene not found in chromosome");
                let mut gene_two = chrom_two.get(i).expect("Gene not found in chromosome");

                while offspring_one[start..=end].contains(gene_one) {
                    let index = chrom_two.genes.iter().position(|g| g == gene_one).unwrap();
                    gene_one = chrom_one.get(index).expect("Gene not found in chromosome");
                }

                while offspring_two[start..=end].contains(gene_two) {
                    let index = chrom_one.genes.iter().position(|g| g == gene_two).unwrap();
                    gene_two = chrom_two.get(index).expect("Gene not found in chromosome");
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
