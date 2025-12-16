use radiate_core::{
    AlterResult, BoundedGene, Chromosome, Crossover, FloatGene, Gene, Rate, Valid, random_provider,
};

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

impl<C: Chromosome<Gene = FloatGene>> Crossover<C> for SimulatedBinaryCrossover {
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
                    let beta = if u <= 0.5 {
                        (2.0 * u).powf(1.0 / (self.contiguty + 1.0))
                    } else {
                        (0.5 / (1.0 - u)).powf(1.0 / (self.contiguty + 1.0))
                    };

                    let v1 = chrom_one.get(i).allele();
                    let v2 = chrom_two.get(i).allele();

                    let v = if rand.bool(0.5) {
                        (v1 - v2) * 0.5 - (beta * 0.5 * (v1 - v2).abs())
                    } else {
                        (v1 - v2) * 0.5 + (beta * 0.5 * (v1 - v2).abs())
                    };

                    let new_gene = v.clamp(*chrom_one.get(i).min(), *chrom_one.get(i).max());

                    count += 1;

                    chrom_one.set(i, chrom_one.get(i).with_allele(&new_gene));
                }
            }
        });

        AlterResult::from(count)
    }
}
