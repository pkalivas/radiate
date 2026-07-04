use radiate_core::{
    AlterContext, AlterResult, BoundedGene, Chromosome, Crossover, Expr, ExprSet, Gene,
    random_provider,
};
use radiate_utils::Float;

const SBX_CROSSOVER_RATE: &str = "crossover.sbx.rate";

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

    fn expressions(&self) -> ExprSet {
        ExprSet::from(self.rate.clone().alias(SBX_CROSSOVER_RATE))
    }

    #[inline]
    fn cross_chromosomes(
        &self,
        chrom_one: &mut C,
        chrom_two: &mut C,
        _: &mut AlterContext,
    ) -> AlterResult {
        let length = std::cmp::min(chrom_one.len(), chrom_two.len());

        if length < 2 {
            return AlterResult::empty();
        }

        let mut count = 0;

        random_provider::with_rng(|rand| {
            let one_slice = chrom_one.as_mut_slice();
            let two_slice = chrom_two.as_slice();
            for i in 0..length {
                if rand.bool(0.5) {
                    let u = rand.random::<f32>();
                    let beta = A::from(if u <= 0.5 {
                        (2.0 * u).powf(1.0 / (self.contiguty + 1.0))
                    } else {
                        (0.5 / (1.0 - u)).powf(1.0 / (self.contiguty + 1.0))
                    })
                    .unwrap();

                    let v1 = *one_slice[i].allele();
                    let v2 = *two_slice[i].allele();

                    let v = if rand.bool(0.5) {
                        ((v1 - v2) * A::HALF) - (beta * A::HALF * (v1 - v2).abs())
                    } else {
                        ((v1 - v2) * A::HALF) + (beta * A::HALF * (v1 - v2).abs())
                    };

                    let (one_min, one_max) = one_slice[i].bounds();
                    let new_gene = v.clamp(*one_min, *one_max);

                    count += 1;

                    *one_slice[i].allele_mut() = new_gene;
                }
            }
        });

        AlterResult::from(count)
    }
}
