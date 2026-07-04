use radiate_core::{AlterContext, AlterResult, Chromosome, Crossover, Expr, Expr, ExprSet, random_provider};

const SHUFFLE_CROSSOVER_RATE: &str = "crossover.shuffle.rate";

pub struct ShuffleCrossover {
    rate: Expr,
}

impl ShuffleCrossover {
    pub fn new(rate: impl Into<Expr>) -> Self {
        ShuffleCrossover { rate: rate.into().alias(SHUFFLE_CROSSOVER_RATE) }
    }
}

impl<C: Chromosome + Clone> Crossover<C> for ShuffleCrossover {
    fn rates(&self) -> ExprSet {
        ExprSet::from(self.rate.clone())
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut C,
        chrom_two: &mut C,
        ctx: &mut AlterContext,
    ) -> AlterResult {
        let length = std::cmp::min(chrom_one.len(), chrom_two.len());
        if length < 2 {
            return AlterResult::empty();
        }

        let mut cross_count = 0;

        random_provider::with_rng(|rand| {
            let mut indices = (0..length).collect::<Vec<usize>>();
            rand.shuffle(&mut indices);

            let temp_chrom_one = chrom_one.as_mut_slice();
            let temp_chrom_two = chrom_two.as_mut_slice();

            for (i, &index) in indices.iter().enumerate() {
                if i % 2 == 0 {
                    if !rand.bool(ctx.rate()) {
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
