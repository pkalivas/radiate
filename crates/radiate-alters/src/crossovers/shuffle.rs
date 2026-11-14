use radiate_core::{
    AlterContext, Crossover, Expr, RateSet, chromosomes::ContiguousChromosome, random_provider,
};

pub struct ShuffleCrossover {
    rate: Expr,
}

impl ShuffleCrossover {
    pub fn new(rate: impl Into<Expr>) -> Self {
        ShuffleCrossover { rate: rate.into() }
    }
}

impl<C: ContiguousChromosome + Clone> Crossover<C> for ShuffleCrossover {
    fn rates(&self) -> RateSet {
        RateSet::new(self.rate.clone())
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut C,
        chrom_two: &mut C,
        ctx: &mut AlterContext,
    ) -> usize {
        let length = std::cmp::min(chrom_one.len(), chrom_two.len());
        if length < 2 {
            return 0;
        }

        let mut cross_count = 0;

        random_provider::with_rng(|rand| {
            let indices = rand.shuffled_indices(0..length);

            for (i, &index) in indices.iter().enumerate() {
                if i % 2 == 0 {
                    if !rand.bool(ctx.rate()) {
                        continue;
                    }

                    let one_gene = chrom_one.get_mut(index);
                    let two_gene = chrom_two.get_mut(index);

                    if let Some((one, two)) = one_gene.zip(two_gene) {
                        std::mem::swap(one, two);
                        cross_count += 1;
                    }
                }
            }
        });

        cross_count
    }
}
