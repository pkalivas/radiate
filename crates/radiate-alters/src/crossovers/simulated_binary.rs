use radiate_core::{
    AlterContext, BoundedGene, Chromosome, Crossover, Expr, Gene, RateSet, random_provider,
};
use radiate_utils::Float;

pub struct SimulatedBinaryCrossover {
    rate: Expr,
    contiguty: f32,
}

impl SimulatedBinaryCrossover {
    pub fn new(rate: impl Into<Expr>, contiguty: f32) -> Self {
        Self {
            rate: rate.into(),
            contiguty,
        }
    }
}

impl<A, G, C> Crossover<C> for SimulatedBinaryCrossover
where
    A: Float,
    G: Gene<Allele = A> + BoundedGene,
    C: Chromosome<Gene = G>,
{
    fn name(&self) -> String {
        "crossover.sbx".to_string()
    }

    fn rates(&self) -> RateSet {
        RateSet::new(self.rate.clone())
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut C,
        chrom_two: &mut C,
        _: &mut AlterContext,
    ) -> usize {
        let length = std::cmp::min(chrom_one.len(), chrom_two.len());

        if length < 2 {
            return 0;
        }

        let mut count = 0;
        random_provider::with_rng(|rand| {
            chrom_one.zip_mut(chrom_two).for_each(|gene_one, gene_two| {
                if rand.bool(0.5) {
                    let u = rand.random::<f32>();
                    let beta = A::from(if u <= 0.5 {
                        (2.0 * u).powf(1.0 / (self.contiguty + 1.0))
                    } else {
                        (0.5 / (1.0 - u)).powf(1.0 / (self.contiguty + 1.0))
                    })
                    .unwrap();

                    let v1 = *gene_one.allele();
                    let v2 = *gene_two.allele();

                    let v = if rand.bool(0.5) {
                        ((v1 - v2) * A::HALF) - (beta * A::HALF * (v1 - v2).abs())
                    } else {
                        ((v1 - v2) * A::HALF) + (beta * A::HALF * (v1 - v2).abs())
                    };

                    let (one_min, one_max) = gene_one.bound_range();
                    let new_gene = v.clamp(*one_min, *one_max);

                    count += 1;

                    *gene_one.allele_mut() = new_gene;
                }
            });
        });

        count
    }
}
