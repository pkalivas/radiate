use radiate_core::{
    AlterResult, BoundedGene, Chromosome, Crossover, FloatGene, Gene, Rate, Valid, random_provider,
};
use radiate_utils::Float;

pub struct SimulatedBinaryCrossover {
    crossover_rate: Rate,
    contiguty: f32,
}

impl SimulatedBinaryCrossover {
    pub fn new(crossover_rate: impl Into<Rate>, contiguty: f32) -> Self {
        let crossover_rate = crossover_rate.into();
        if !crossover_rate.is_valid() {
            panic!("Rate {crossover_rate:?} is not valid. Must be between 0.0 and 1.0",);
        }
        Self {
            contiguty,
            crossover_rate,
        }
    }
}

impl<F, C> Crossover<C> for SimulatedBinaryCrossover
where
    F: Float,
    C: Chromosome<Gene = FloatGene<F>>,
{
    fn rate(&self) -> Rate {
        self.crossover_rate.clone()
    }

    #[inline]
    fn cross_chromosomes(&self, chrom_one: &mut C, chrom_two: &mut C, _: f32) -> AlterResult {
        let length = std::cmp::min(chrom_one.len(), chrom_two.len());

        if length < 2 {
            return AlterResult::empty();
        }

        let mut count = 0;

        random_provider::with_rng(|rand| {
            for i in 0..length {
                if rand.bool(0.5) {
                    let u = rand.random::<f32>();
                    let beta = F::from(if u <= 0.5 {
                        (2.0 * u).powf(1.0 / (self.contiguty + 1.0))
                    } else {
                        (0.5 / (1.0 - u)).powf(1.0 / (self.contiguty + 1.0))
                    })
                    .unwrap();

                    let v1 = chrom_one.get(i).allele().clone();
                    let v2 = chrom_two.get(i).allele().clone();

                    let v = if rand.bool(0.5) {
                        (v1 - v2) * F::HALF - (beta * F::HALF * (v1 - v2).abs())
                    } else {
                        (v1 - v2) * F::HALF + (beta * F::HALF * (v1 - v2).abs())
                    };

                    let (one_min, one_max) = chrom_one.get(i).bounds();
                    let new_gene = v.clamp(*one_min, *one_max);

                    count += 1;

                    *chrom_one.get_mut(i).allele_mut() = new_gene;
                }
            }
        });

        AlterResult::from(count)
    }
}
